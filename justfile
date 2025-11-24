# Update local main branch
new:
    git checkout main && git fetch && git pull origin main

# Run BIOS boot test (default: SCPH1001.BIN, 100000 instructions)
bios BIOS="SCPH1001.BIN" INSTRUCTIONS="100000":
    cargo run --bin psrx-ui {{BIOS}} -n {{INSTRUCTIONS}}

# Run BIOS boot test in release mode
bios-release BIOS="SCPH1001.BIN" INSTRUCTIONS="100000":
    cargo run --release --bin psrx-ui {{BIOS}} -n {{INSTRUCTIONS}}

# Run game with BIOS and CUE file (default: SCPH1001.BIN)
run BIOS="SCPH1001.BIN" CUE="" INSTRUCTIONS="1000000":
    cargo run --bin psrx-ui {{BIOS}} --cdrom {{CUE}} -n {{INSTRUCTIONS}}

# Run game in release mode with BIOS and CUE file
run-release BIOS="SCPH1001.BIN" CUE="" INSTRUCTIONS="1000000":
    cargo run --release --bin psrx-ui {{BIOS}} --cdrom {{CUE}} -n {{INSTRUCTIONS}}