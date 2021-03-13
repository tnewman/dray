# 🚚 Dray 🚚
A cloud native SFTP server designed to support multiple data storage backends, starting with S3.

## What's With the Name
A dray is a cart used to transport heavy cargo short distances. Dray transports files of any size 
to the storage backend.

## 🚧 Work in Progress 🚧
This project is currently not in a usable state. The project will be considered usable when 
the MVP roadmap has been implemented.

## Minimum Viable Product (MVP) Roadmap
- [x] Deserialize and Serialize SSH File Transfer Protocol Version 3 Draft 2
- [x] Accept SSH connections
- [x] Verify SSH keys against authorized keys stored in S3
- [] SFTP subsystem initialization
- [] List directory (S3-Compatible Storage Only)
- [] Create directory (S3-Compatible Storage Only)
- [] Rename directory (S3-Compatible Storage Only)
- [] Remove directory (S3-Compatible Storage Only)
- [] Read file (S3-Compatible Storage Only)
- [] Write file (S3-Compatible Storage Only)
- [] Rename file (S3-Compatible Storage Only)
- [] Remove file (S3-Compatible Storage Only)
- [] No-Op/Defaults for other SFTP commands
