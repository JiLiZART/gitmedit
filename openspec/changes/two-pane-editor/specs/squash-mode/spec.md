## REMOVED Requirements

### Requirement: Squash operation is recognized
**Reason**: Squash messages use the message layout; the header split is replaced by the general
message/trailer split.
**Migration**: See `text-editing` "Message files are split into message and trailer".

### Requirement: Commit log is shown read-only
**Reason**: Comment lines, including the squash notes, are shown in the status pane instead of a
read-only area in the editor.
**Migration**: See `status-pane` "Comment block parsed into sections" (Squash section).

### Requirement: Combined message is editable
**Reason**: Covered by the shared message editor.
**Migration**: See `text-editing` "Full text editing".

### Requirement: Generated content is preserved on save
**Reason**: Superseded by trailer reassembly, which preserves every comment line.
**Migration**: See `text-editing` "Save reassembles the file".
