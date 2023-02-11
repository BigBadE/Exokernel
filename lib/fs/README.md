# Disclaimer

This crate isn't used! Instead, FAT is used because this FS doesn't work.
It's entirely for learning, and may be tested in the future.

# File System

Linked-list filesystem with emphasis on fast iteration.

# Design

- Files are stored sequentially if possible, to minimize seeking.
- Read only mode allows no heap usage for early kernel stage use.
- Protect against sector failure in tree structure with built-in redundancy.
- Allows for non-sector size clusters.

# Header
Partitions start with the header 0xCA5CADE as a sanity check. 
Next, it has one byte which is the sector size (found by 2^x).

Next is the root directory.

# Directories
| Offset | Size | Description                       |
|--------|:----:|-----------------------------------|
| 0x000  |  8   | Start of first child file/folder  |
| 0x008  |  8   | Start of next file/folder         |
| 0x010  |  2   | Security bytes                    |
| 0x012  | 256  | Directory name                    |
| 0x112  |  8   | Start of parent folder            |
| 0x11A  |  8   | Start of second child file/folder |
| 0x122  |  8   | Start of second next file/folder  |

# Files
| Offset | Size | Description               |
|--------|:----:|---------------------------|
| 0x000  |  8   | Start of next file        |
| 0x008  |  2   | Security bytes            |
| 0x00A  |  8   | Start of parent folder    |
| 0x012  |  8   | Start of second next file |
| 0x01A  |  8   | File size                 |
| 0x022  | 256  | File name                 |
| 0x122  |  -   | File contents             |
