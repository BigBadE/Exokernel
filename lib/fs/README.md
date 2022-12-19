# File System

Linked-list filesystem with emphasis on fast iteration.

# Design

- Disk is divided into 10 bands. Each band starts and ends with a file table.
- The "bottom" band grows upwards, and the "top" band grows downwards.
- Files are stored sequentially if possible, to minimize seeking.
- Read only mode allows no heap usage for early kernel stage use.

# Header
Partitions start with the header 0xCA5CADE

# Bands
There are 10 bands total, each with two band file tables.
Bands are placed at exact intervals along the drive, so no master file table is needed.
Band file tables are identical, with two copies at the start of each band.
Each entry starts with an IDENTITY byte which determines how it's parsed.
Each sector ends with the sector the table continues at. Since both tables should be in sync,
the exact position to read to for syncing should be known.

IDENTITY 0x0: File:
Offset | Size | Description
0x0    | 8    | Sector of the file 
0x8    | 2    | Security flags of the file
0xA    | 1    | File information flags
0xB    | 32   | First 8 characters of file name
0x2B   | 8    | First 2 characters of file extension

IDENTITY 0x1: Directory:
Offset | Size | Description
0x0    | 8    | Sector of the file
0x8    | 2    | Security flags of the file
0xA    | 1    | Directory information flags
0xB    | 32   | First 8 characters of file name

IDENTITY 0x2: Bad Sector Table:
These are expected to be loaded into memory if the READ_ONLY flag isn't set.

Offset | Size | Description
0x0    | 8    | Bad sector start
0x8    | 8    | Bad sector end

IDENTITY 0x3: Allocation Range:
These are expected to be loaded into memory if the READ_ONLY flag isn't set.

Offset | Size | Description
0x0    | 8    | Allocation sector range start
0x8    | 8    | Allocation sector range end

# File information flags
Offset | Description
0x0    | 0 if the file is not continuous, 1 if it is

# Directory information flags
Offset | Description

# File reading
Files are stored raw at the given sector. The first up to 1028 bytes of the file are the file name, null terminated.

At the end of each sector is 8 bytes pointing to the next part if the file is not continuous. It may
be the next sector even if the file is not continuous.

# Directory reading
Directories have a copy of the IDENTITY 0x0 or IDENTITY 0x1 table for each file and directory in it.

# Sector failure
Detected failed sectors are added to the band table. 
Should be checked if the sectors are at the start/end of another bad sector range, and if so, extend that range instead.

Write operations that run into a bad sector won't error, and will instead silently look for a good sector to replace it.
Read operations that run into a bad sector will error.

# File allocation
First, all bands will be scanned to find any ranges that will fit the file.
If multiple bigger ranges are found, and no exact fits are found, the smallest will be chosen.