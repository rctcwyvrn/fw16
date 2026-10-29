# synth: synthesis only (Phase 2 stops before implementation). Writes the
# utilisation and timing summary into <build>/reports/. Timing here uses
# synthesis delay estimates; treat WNS as a first look, not a sign-off.

source [file join [file dirname [info script]] settings.tcl]

open_project [file join $omni_projdir $omni_proj.xpr]
reset_run synth_1
launch_runs synth_1 -jobs 8
wait_on_run synth_1
if {[get_property PROGRESS [get_runs synth_1]] ne "100%"} {
  error "synth_1 failed: see [get_property DIRECTORY [get_runs synth_1]]/runme.log"
}

open_run synth_1
report_utilization -file [file join $omni_reports synth_utilization.txt]
report_utilization -hierarchical -hierarchical_depth 2 \
  -file [file join $omni_reports synth_utilization_hier.txt]
report_timing_summary -max_paths 10 -file [file join $omni_reports synth_timing_summary.txt]
report_clocks -file [file join $omni_reports synth_clocks.txt]
puts "omni: synth reports in $omni_reports"
close_project
