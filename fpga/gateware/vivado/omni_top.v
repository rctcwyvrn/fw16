// omni_top: XDMA (Gen4 x4, 256-bit AXI-ST at 250 MHz) wired to the Hardcaml
// shell. The shell runs entirely on XDMA's axi_aclk; its clear is the
// active-high, synchronous inverse of axi_aresetn.
//
// The XDMA instance is named xdma_0_i, as in AMD's example design, because
// sim_bfm.tcl renames this module to xilinx_dma_pcie_ep and drops it into the
// example testbench, which may reach into EP.xdma_0_i hierarchically.

`timescale 1ps / 1ps

module omni_top (
    input        sys_clk_p,
    input        sys_clk_n,
    input        sys_rst_n,
    output [3:0] pci_exp_txp,
    output [3:0] pci_exp_txn,
    input  [3:0] pci_exp_rxp,
    input  [3:0] pci_exp_rxn
);

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
        .s_axis_c2h_tready_0 (s_axis_c2h_tready_0)
    );

    omni_shell shell (
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
        .usr_irq_req (usr_irq_req[0])
    );

endmodule
