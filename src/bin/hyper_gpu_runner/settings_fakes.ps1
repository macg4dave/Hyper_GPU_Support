# Test-only native cmdlet fakes. Privilege: non-elevated; no actual Windows effects.
$script:memory = [pscustomobject]@{Startup=[uint64]4294967296; DynamicMemoryEnabled=$true}
$script:cpu = [pscustomobject]@{Count=2; ExposeVirtualizationExtensions=$false}
$script:vm = [pscustomobject]@{
    Id=$vmId; Name=$vmName; State='Off'; Generation=2; Version='12.0'
    AutomaticCheckpointsEnabled=$false; CheckpointType='Production'; AutomaticStopAction='Save'
    LowMemoryMappedIoSpace=[uint64]1073741824; HighMemoryMappedIoSpace=[uint64]2147483648; GuestControlledCacheTypes=$false
}
$script:adapter = [pscustomobject]@{InstancePath=$gpuPath}
$script:hostGpu = [pscustomobject]@{Name=$gpuPath}
foreach ($n in @('VRAM','Encode','Decode','Compute')) {
    foreach ($prefix in @('Min','Max','Optimal')) {
        $script:adapter | Add-Member NoteProperty ($prefix+'Partition'+$n) ([uint64]1)
        $limit = if ($prefix -eq 'Min') { [uint64]0 } else { [uint64]::MaxValue }
        $script:hostGpu | Add-Member NoteProperty ($prefix+'Partition'+$n) $limit
    }
}
function Get-VM { param($Id,$ErrorAction) $script:vm.PSObject.Copy() }
function Get-VMMemory { param($VM,$ErrorAction) $script:memory.PSObject.Copy() }
function Get-VMProcessor { param($VM,$ErrorAction) $script:cpu.PSObject.Copy() }
function Get-VMFirmware { param($VM,$ErrorAction) [pscustomobject]@{SecureBoot='On'; SecureBootTemplate='MicrosoftWindows'} }
function Get-VMSecurity { param($VM,$ErrorAction) [pscustomobject]@{TpmEnabled=$true} }
function Get-VMGpuPartitionAdapter { param($VM,$ErrorAction) $script:adapter.PSObject.Copy() }
function Get-VMHostPartitionableGpu { param($Name,$ErrorAction) $script:hostGpu.PSObject.Copy() }
function Get-VMSnapshot { param($VM,$ErrorAction) }
function Get-VMHardDiskDrive { param($VM,$ErrorAction) [pscustomobject]@{Path=$childPath} }
function Get-Item { param($LiteralPath,$ErrorAction) [pscustomobject]@{IsReadOnly=$true; Attributes=[IO.FileAttributes]0} }
function Get-VHD { param($Path,$ErrorAction) [pscustomobject]@{Path=$childPath; ParentPath=$parentPath; VhdType='Differencing'} }
function Set-VM {
    param($VM,[uint64]$LowMemoryMappedIoSpace,[uint64]$HighMemoryMappedIoSpace,[bool]$GuestControlledCacheTypes,$CheckpointType,[bool]$AutomaticCheckpointsEnabled,$AutomaticStopAction,$ErrorAction)
    foreach ($key in @('LowMemoryMappedIoSpace','HighMemoryMappedIoSpace','GuestControlledCacheTypes','CheckpointType','AutomaticCheckpointsEnabled','AutomaticStopAction')) { $script:vm.$key = $PSBoundParameters[$key] }
}
function Set-VMMemory {
    param($VM,[bool]$DynamicMemoryEnabled,[uint64]$StartupBytes,$ErrorAction)
    $script:memory.Startup=$StartupBytes; $script:memory.DynamicMemoryEnabled=$DynamicMemoryEnabled
}
function Set-VMProcessor {
    param($VM,[uint32]$Count,[bool]$ExposeVirtualizationExtensions,$ErrorAction)
    $script:cpu.Count=$Count; $script:cpu.ExposeVirtualizationExtensions=$ExposeVirtualizationExtensions
}
function Set-VMGpuPartitionAdapter {
    param($VM,[uint64]$MinPartitionVRAM,[uint64]$MaxPartitionVRAM,[uint64]$OptimalPartitionVRAM,
        [uint64]$MinPartitionEncode,[uint64]$MaxPartitionEncode,[uint64]$OptimalPartitionEncode,
        [uint64]$MinPartitionDecode,[uint64]$MaxPartitionDecode,[uint64]$OptimalPartitionDecode,
        [uint64]$MinPartitionCompute,[uint64]$MaxPartitionCompute,[uint64]$OptimalPartitionCompute,$ErrorAction)
    if ($script:failGpu) { throw 'injected partial GPU update failure' }
    foreach ($key in @($PSBoundParameters.Keys)) { if ($key -like '*Partition*') { $script:adapter.$key=$PSBoundParameters[$key] } }
}
$script:failGpu=$false
