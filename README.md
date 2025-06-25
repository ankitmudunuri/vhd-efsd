# VHD Encryption and Fragmentation Self-Destructable Project
This project is a Rust app that lets you securely store information in a VHD. It has a login system, but behind that it fragments the drive into multiple binary files with binary chunks of the VHD, as well as encrypting it beforehand, securing the data that is within from unauthorized access. This is still a work-in-progress (and also a shitty explanation), so I will be finishing the final feature of this and updating the readme in the future. Feel free to reach out to me if you want to contribute or discuss anything about the project!

Final features yet to be made:
- self-destruction of all fragments and supporting files after 5 failed login attempts
- TPM support to hold a master key for encryption/decryption (master key will be combined with user password to create an encryption key)


## Tutorial

When you launch the app, it will ask you to enter a password to get started. Once you do that, it will ask for the amount of files and binary chunks. This is for the AKIFA Algorithm, which is explained in [AKIFA Overview](AKIFA_Overview.md). It will then ask for a separate encryption password, so enter that (this will encrypt the drive with this password before fragmenting it). Once you do that, it will also ask how big you want the drive to be. Enter that, and the letter that you want to assign to your drive, and it should be all good from there!

If you want to encrypt and fragment the drive after use, then just run the app again and enter the login and encryption password, and it will unmount, encrypt, and fragment the VHD file. 

If you want to access the VHD again, run the app, enter the login and encryption passwords again and it will automatically reassemble and decrypt the VHD, as well as mounting it.

(This README is incomplete right now, I will finish it later).