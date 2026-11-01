// fw16_fpga_top: XDMA (Gen4 x4, 256-bit AXI-ST at 250 MHz) wired to the Hardcaml
// shell. The shell runs entirely on XDMA's axi_aclk; its clear is the
// active-high, synchronous inverse of axi_aresetn. The shell's project slot gets
// XDMA's BAR2 AXI4 bypass master and clk100m as its free slot_clk (decision 0013).
//
// The XDMA instance is named xdma_0_i, as in AMD's example design, because
// sim_bfm.tcl renames this module to xilinx_dma_pcie_ep and drops it into the
// example testbench, which may reach into EP.xdma_0_i hierarchically.

`timescale 1ps / 1ps

module fw16_fpga_top (
    input        sys_clk_p,
    input        sys_clk_n,
    input        sys_rst_n,
    output [3:0] pci_exp_txp,
    output [3:0] pci_exp_txn,
    input  [3:0] pci_exp_rxp,
    input  [3:0] pci_exp_rxn,

    // Card I/O on HD bank 84 (3.3 V), pins from bay/card/fpga/pinplan.csv via fw16_fpga_pins.xdc.
    // Safe defaults until the shell owns them: nothing here drives a shared line.
    input        clk100m,       // 100 MHz LVCMOS oscillator (Y1)
    output       led_hb,        // heartbeat LED, ~0.75 Hz from clk100m
    output       led_link,      // PCIe link-up LED
    output       th_alert,      // drives Q1 -> TH_ALERT# (open drain to the host); held low = not asserted
    output       th_overt,      // drives Q2 -> TH_OVERT# (PROCHOT#); held low = not asserted
    inout        sense_scl,     // INA237 bus, pulled up on the card; released
    inout        sense_sda,
    input        sense_alert_n, // INA237 ALERT, unused yet
    input        pps_in,        // U.FL PPS / 10 MHz ref / trigger; synchronised inside the shell
    inout        clkreq_n       // only via DNP R24; released
);

    // Heartbeat on the free-running fabric clock, independent of PCIe.
    reg [26:0] hb_count = 27'd0;
    always @(posedge clk100m) hb_count <= hb_count + 27'd1;
    assign led_hb = hb_count[26];

    assign th_alert  = 1'b0;
    assign th_overt  = 1'b0;
    assign sense_scl = 1'bz;
    assign sense_sda = 1'bz;
    assign clkreq_n  = 1'bz;

    // 100 MHz PCIe reference: ODIV2 for the core, O for the GTs.
    wire sys_clk;
    wire sys_clk_gt;
    wire sys_rst_n_c;

    IBUFDS_GTE4 #(.REFCLK_HROW_CK_SEL(2'b00)) refclk_ibuf (
        .O(sys_clk_gt), .ODIV2(sys_clk), .I(sys_clk_p), .IB(sys_clk_n), .CEB(1'b0));
    IBUF sys_reset_n_ibuf (.O(sys_rst_n_c), .I(sys_rst_n));

    wire axi_aclk;
    wire axi_aresetn;
    wire user_lnk_up;
    assign led_link = user_lnk_up;

    // AXI-Lite master (BAR0) -> shell registers
    wire [31:0] m_axil_awaddr;
    wire        m_axil_awvalid, m_axil_awready;
    wire [31:0] m_axil_wdata;
    wire [3:0]  m_axil_wstrb;
    wire        m_axil_wvalid, m_axil_wready;
    wire [1:0]  m_axil_bresp;
    wire        m_axil_bvalid, m_axil_bready;
    wire [31:0] m_axil_araddr;
    wire        m_axil_arvalid, m_axil_arready;
    wire [31:0] m_axil_rdata;
    wire [1:0]  m_axil_rresp;
    wire        m_axil_rvalid, m_axil_rready;

    // H2C channel 0 (XDMA master) -> shell sink / ring
    wire [255:0] m_axis_h2c_tdata_0;
    wire [31:0]  m_axis_h2c_tkeep_0;
    wire         m_axis_h2c_tlast_0, m_axis_h2c_tvalid_0, m_axis_h2c_tready_0;

    // Shell ring -> C2H channel 0 (XDMA slave)
    wire [255:0] s_axis_c2h_tdata_0;
    wire [31:0]  s_axis_c2h_tkeep_0;
    wire         s_axis_c2h_tlast_0, s_axis_c2h_tvalid_0, s_axis_c2h_tready_0;

    wire [0:0] usr_irq_req;
    wire [0:0] usr_irq_ack;

    // AXI4 bypass master (BAR2, 1 MiB, addr = BAR2 offset) -> the shell's project slot
    wire [3:0]   m_axib_awid,    m_axib_arid,    m_axib_bid,    m_axib_rid;
    wire [63:0]  m_axib_awaddr,  m_axib_araddr;
    wire [7:0]   m_axib_awlen,   m_axib_arlen;
    wire [2:0]   m_axib_awsize,  m_axib_arsize,  m_axib_awprot, m_axib_arprot;
    wire [1:0]   m_axib_awburst, m_axib_arburst, m_axib_bresp,  m_axib_rresp;
    wire [3:0]   m_axib_awcache, m_axib_arcache;
    wire         m_axib_awlock,  m_axib_arlock;
    wire         m_axib_awvalid, m_axib_awready, m_axib_arvalid, m_axib_arready;
    wire [255:0] m_axib_wdata,   m_axib_rdata;
    wire [31:0]  m_axib_wstrb;
    wire         m_axib_wlast,   m_axib_wvalid,  m_axib_wready;
    wire         m_axib_bvalid,  m_axib_bready;
    wire         m_axib_rlast,   m_axib_rvalid,  m_axib_rready;

    xdma_0 xdma_0_i (
        .sys_clk     (sys_clk),
        .sys_clk_gt  (sys_clk_gt),
        .sys_rst_n   (sys_rst_n_c),
        .pci_exp_txp (pci_exp_txp),
        .pci_exp_txn (pci_exp_txn),
        .pci_exp_rxp (pci_exp_rxp),
        .pci_exp_rxn (pci_exp_rxn),

        .axi_aclk    (axi_aclk),
        .axi_aresetn (axi_aresetn),
        .user_lnk_up (user_lnk_up),

        .usr_irq_req (usr_irq_req),
        .usr_irq_ack (usr_irq_ack),

        .m_axil_awaddr  (m_axil_awaddr),
        .m_axil_awprot  (),
        .m_axil_awvalid (m_axil_awvalid),
        .m_axil_awready (m_axil_awready),
        .m_axil_wdata   (m_axil_wdata),
        .m_axil_wstrb   (m_axil_wstrb),
        .m_axil_wvalid  (m_axil_wvalid),
        .m_axil_wready  (m_axil_wready),
        .m_axil_bvalid  (m_axil_bvalid),
        .m_axil_bresp   (m_axil_bresp),
        .m_axil_bready  (m_axil_bready),
        .m_axil_araddr  (m_axil_araddr),
        .m_axil_arprot  (),
        .m_axil_arvalid (m_axil_arvalid),
        .m_axil_arready (m_axil_arready),
        .m_axil_rdata   (m_axil_rdata),
        .m_axil_rresp   (m_axil_rresp),
        .m_axil_rvalid  (m_axil_rvalid),
        .m_axil_rready  (m_axil_rready),

        .m_axis_h2c_tdata_0  (m_axis_h2c_tdata_0),
        .m_axis_h2c_tkeep_0  (m_axis_h2c_tkeep_0),
        .m_axis_h2c_tlast_0  (m_axis_h2c_tlast_0),
        .m_axis_h2c_tvalid_0 (m_axis_h2c_tvalid_0),
        .m_axis_h2c_tready_0 (m_axis_h2c_tready_0),

        .s_axis_c2h_tdata_0  (s_axis_c2h_tdata_0),
        .s_axis_c2h_tkeep_0  (s_axis_c2h_tkeep_0),
        .s_axis_c2h_tlast_0  (s_axis_c2h_tlast_0),
        .s_axis_c2h_tvalid_0 (s_axis_c2h_tvalid_0),
        .s_axis_c2h_tready_0 (s_axis_c2h_tready_0),

        .m_axib_awid    (m_axib_awid),
        .m_axib_awaddr  (m_axib_awaddr),
        .m_axib_awlen   (m_axib_awlen),
        .m_axib_awsize  (m_axib_awsize),
        .m_axib_awburst (m_axib_awburst),
        .m_axib_awprot  (m_axib_awprot),
        .m_axib_awvalid (m_axib_awvalid),
        .m_axib_awready (m_axib_awready),
        .m_axib_awlock  (m_axib_awlock),
        .m_axib_awcache (m_axib_awcache),
        .m_axib_wdata   (m_axib_wdata),
        .m_axib_wstrb   (m_axib_wstrb),
        .m_axib_wlast   (m_axib_wlast),
        .m_axib_wvalid  (m_axib_wvalid),
        .m_axib_wready  (m_axib_wready),
        .m_axib_bid     (m_axib_bid),
        .m_axib_bresp   (m_axib_bresp),
        .m_axib_bvalid  (m_axib_bvalid),
        .m_axib_bready  (m_axib_bready),
        .m_axib_arid    (m_axib_arid),
        .m_axib_araddr  (m_axib_araddr),
        .m_axib_arlen   (m_axib_arlen),
        .m_axib_arsize  (m_axib_arsize),
        .m_axib_arburst (m_axib_arburst),
        .m_axib_arprot  (m_axib_arprot),
        .m_axib_arvalid (m_axib_arvalid),
        .m_axib_arready (m_axib_arready),
        .m_axib_arlock  (m_axib_arlock),
        .m_axib_arcache (m_axib_arcache),
        .m_axib_rid     (m_axib_rid),
        .m_axib_rdata   (m_axib_rdata),
        .m_axib_rresp   (m_axib_rresp),
        .m_axib_rlast   (m_axib_rlast),
        .m_axib_rvalid  (m_axib_rvalid),
        .m_axib_rready  (m_axib_rready)
    );

    fw16_fpga_shell shell (
        .clock (axi_aclk),
        .clear (~axi_aresetn),

        .s_axil_awvalid (m_axil_awvalid),
        .s_axil_awaddr  (m_axil_awaddr),
        .s_axil_wvalid  (m_axil_wvalid),
        .s_axil_wdata   (m_axil_wdata),
        .s_axil_wstrb   (m_axil_wstrb),
        .s_axil_bready  (m_axil_bready),
        .s_axil_arvalid (m_axil_arvalid),
        .s_axil_araddr  (m_axil_araddr),
        .s_axil_rready  (m_axil_rready),
        .s_axil_awready (m_axil_awready),
        .s_axil_wready  (m_axil_wready),
        .s_axil_bvalid  (m_axil_bvalid),
        .s_axil_bresp   (m_axil_bresp),
        .s_axil_arready (m_axil_arready),
        .s_axil_rvalid  (m_axil_rvalid),
        .s_axil_rdata   (m_axil_rdata),
        .s_axil_rresp   (m_axil_rresp),

        .s_axis_h2c_tvalid (m_axis_h2c_tvalid_0),
        .s_axis_h2c_tdata  (m_axis_h2c_tdata_0),
        .s_axis_h2c_tkeep  (m_axis_h2c_tkeep_0),
        .s_axis_h2c_tlast  (m_axis_h2c_tlast_0),
        .h2c_tready        (m_axis_h2c_tready_0),

        .m_axis_c2h_tvalid (s_axis_c2h_tvalid_0),
        .m_axis_c2h_tdata  (s_axis_c2h_tdata_0),
        .m_axis_c2h_tkeep  (s_axis_c2h_tkeep_0),
        .m_axis_c2h_tlast  (s_axis_c2h_tlast_0),
        .c2h_tready        (s_axis_c2h_tready_0),

        .link_up     (user_lnk_up),
        .temp        (16'd0),  // SYSMON: alive-bitstream step
        .pwr_mw      (32'd0),  // power monitor: not on the card yet
        .usr_irq_ack (usr_irq_ack[0]),
        .pps_in      (pps_in),
        .usr_irq_req (usr_irq_req[0]),

        // Project slot: free clock (no shell logic on it) and the BAR2 bypass port.
        .slot_clk       (clk100m),
        .s_axib_awid    (m_axib_awid),
        .s_axib_awaddr  (m_axib_awaddr),
        .s_axib_awlen   (m_axib_awlen),
        .s_axib_awsize  (m_axib_awsize),
        .s_axib_awburst (m_axib_awburst),
        .s_axib_awprot  (m_axib_awprot),
        .s_axib_awlock  (m_axib_awlock),
        .s_axib_awcache (m_axib_awcache),
        .s_axib_awvalid (m_axib_awvalid),
        .s_axib_wdata   (m_axib_wdata),
        .s_axib_wstrb   (m_axib_wstrb),
        .s_axib_wlast   (m_axib_wlast),
        .s_axib_wvalid  (m_axib_wvalid),
        .s_axib_bready  (m_axib_bready),
        .s_axib_arid    (m_axib_arid),
        .s_axib_araddr  (m_axib_araddr),
        .s_axib_arlen   (m_axib_arlen),
        .s_axib_arsize  (m_axib_arsize),
        .s_axib_arburst (m_axib_arburst),
        .s_axib_arprot  (m_axib_arprot),
        .s_axib_arlock  (m_axib_arlock),
        .s_axib_arcache (m_axib_arcache),
        .s_axib_arvalid (m_axib_arvalid),
        .s_axib_rready  (m_axib_rready),
        .s_axib_awready (m_axib_awready),
        .s_axib_wready  (m_axib_wready),
        .s_axib_bid     (m_axib_bid),
        .s_axib_bresp   (m_axib_bresp),
        .s_axib_bvalid  (m_axib_bvalid),
        .s_axib_arready (m_axib_arready),
        .s_axib_rid     (m_axib_rid),
        .s_axib_rdata   (m_axib_rdata),
        .s_axib_rresp   (m_axib_rresp),
        .s_axib_rlast   (m_axib_rlast),
        .s_axib_rvalid  (m_axib_rvalid)
    );

endmodule
