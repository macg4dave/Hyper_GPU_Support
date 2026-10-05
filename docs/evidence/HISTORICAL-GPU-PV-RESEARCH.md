# Historical GPU-PV investigation

Historical research is closed. The [validated normal-Hyper-V baseline](GPU-PV-BASELINE.md)
and current architecture replace all former comparison procedures and feasibility gates.

Earlier 217-file package/alias staging did not initialize the device reliably.
CUDA/NVML aliases and device cycling did not fix that baseline. A direct HCS update
to the VMMS-managed guest was rejected asynchronously with 0x8004102B; no HCS
product path is scheduled. The complete files/settings recipe later passed all
essential workloads, without isolating individual causes.

Historical: the initial provisioning recipe was derived during reference research
and independently validated on the target system. No external repository lookup,
reference artifact, media modification, guest agent or compatibility shim is needed
for the current product. Original raw run filenames and the measured inventory are
retained in the project baseline. Source authorship provenance remains with the
relevant source; actual shipped component notices remain a package responsibility.
