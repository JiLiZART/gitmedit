## REMOVED Requirements

### Requirement: Conflict markers are detected and styled
**Reason**: Merge messages no longer have a mode of their own; they use the message layout.
**Migration**: Conflict markers are highlighted by `text-editing` "Conflict markers are highlighted".

### Requirement: The resolved message is editable
**Reason**: Every line of a message is editable in the message layout.
**Migration**: See `text-editing` "Full text editing".

### Requirement: Conflict markers are editable
**Reason**: Conflict markers are ordinary message lines; comments leave the editor for the status
pane.
**Migration**: See `text-editing` "Conflict markers are highlighted" and "Save reassembles the file".
