# Alphanumeric Key-Based Incrementational Fragmentation Algorithm (AKIFA)

## Goal
#### The goal of this algorithm is to create a key for fragmentation and reassembly of files in a way where it cannot just be deciphered by adversaries.

## Setup
Obviously before all of this, you will need a file that you want to fragment/reassemble (in this app's case, a VHD file).
The first thing you're going to want to do is to get two variables: <p align="center">
*n*: the number of files to split VHD amongst, *c*: the amount of chunks per file
</p>

## Getting names for file
The next step is to generate random length alphanumeric (a..z,A..Z) strings that will represent filenames. For instance, if you have 6 files and 4 chunks then you will generate 6 strings (1 for each file), with each string's length being less than the max amount of chunks. 

## Generating the key
After the previous step, now you must generate the key to fragment the file with. To help envision this, think of every filename string as a front-end queue, with each character being its own element in that queue. On random, a filename is chosen, and the character at the front of the queue is popped and added to the key's string. After this, another filename is randomly chosen, and the character at the front is popped and added to the key. This is done until all of the filenames' symbolic queues have no more elements.

An example of this:

n = 3, c = 4

File 1: a3os, File 2: bdz, File 3: 9

Step 1: File 2 is chosen, key = b, File 1: a3os, File 2: dz, File 3: 9  
Step 2: File 3 is chosen, key = b9, File 1: a3os, File 2: dz, File 3:   
Step 3: File 1 is chosen, key = b9a, File 1: 3os, File 2: dz, File 3:   
Step 4: File 1 is chosen, key = b9a3, File 1: os, File 2: dz, File 3:   
Step 5: File 2 is chosen, key = b9a3d, File 1: os, File 2: z, File 3:  
Step 6: File 1 is chosen, key = b9a3do, File 1: s, File 2: z, File 3:  
Step 7: File 1 is chosen, key = b9a3dos, File 1: , File 2: z, File 3:  
Step 8: File 2 is chosen, key = b9a3dosz, File 1: , File 2: , File 3:  


## Resolving issues with key generation
A rising issue with this key generation algorithm is if two files have the same character at the front of their queue (ex. File 1: ahuiebg, File 2: ldehlsng, files get picked to the point where the character "h" is the one at the front of both queues). One way to resolve this is to increment/decrement the ASCII by one on one of the filenames, chosen randomly. Though inefficient as of now, the algorithm iterates through the entire list of queues to make sure there are no conflicts, iterating as it does. One way to make this more efficient is to have a set that holds the front values of the previous queues checked, and then iterate if the character is in the set (but I was too lazy to implement it as of the time of writing this). 

The other issue with incrementing is if you get to the ends of the alphabet and increment past that. If you have some ASCII knowledge, you know that past those ends of the alphabet, for both upper and lowercase as well as numbers, is control values and punctuation. This would mess up filenames and paths, so I refrained from using that and instead made it so that it just wraps around (so a - 1 = Z, and z + 1 = A, and so on and so forth).

## Fragmentation
The final step is to then split the binary data of the file by the length of the key, creating even-sized chunks (example: if len = 10, and the size of the file is 10MB, then each chunk is 1MB). Then, add the chunks to the binary of each file in the order that their filenames shows up in the key.

Example:  
Key = ueb3894nf, File 1: eb84n, File 2: u39f  
Since len(Key) = 9, File 1 gets chunks [2, 3, 5, 7, 8], while File 2 gets chunks [1, 4, 6, 9].

These files are then, in the case of this app, spread across to random areas of the PC to be hidden and stored. The directories are then mapped in a separately encrypted file.

## Reassembly
Using the fragmentation process, it is possible to reassemble the files doing the exact reverse of fragmentation. Once the key is obtained, and the files and their respective binary chunks loaded, you can just parse the order of the key and essentially "pop" the binary chunk from the front of the respective file.