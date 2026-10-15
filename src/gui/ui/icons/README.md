# Prototype icons

These SVGs were drawn locally for this prototype; no external artwork or
third-party icon package was imported. They depict a four-pane Windows-style
emblem, a Tux-inspired Linux penguin, a generic monitor, and disclosure chevrons.
OS symbols use guest-reported OS names read from Hyper-V intrinsic KVP metadata,
keyed by VM GUID. Names of VMs are never used to infer their OS. Missing, ambiguous,
unreadable or unrecognised metadata retains the generic monitor icon. This is optional
host inventory; no guest is started or accessed to obtain an icon. The symbols imply
no vendor endorsement. Monochrome icons use the current foreground palette;
the penguin retains contrasting light details and an outline in both themes.
