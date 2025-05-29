# VHD Encryption and Fragmentation Self-Destructable Project
This project is a Rust app that lets you securely store information in a VHD. It has a login system, but behind that it fragments the drive into multiple binary files with binary chunks of the VHD, as well as encrypting it beforehand, securing the data that is within from unauthorized access. This is still a work-in-progress (and also a shitty explanation), so I will be finishing the final feature of this and updating the readme in the future.

Final features yet to be made:
- self-destruction of all fragments and supporting files after 5 failed login attempts
- TPM support to hold a master key for encryption/decryption (master key will be combined with user password to create an encryption key)
