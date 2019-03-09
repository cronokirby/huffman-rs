# Huffman
This is a little command line program using [Huffman Coding](https://en.wikipedia.org/wiki/Huffman_coding)
to compress and decompress files

## Usage
```
USAGE:
    huffman <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    decode    Decode a file
    encode    Encode a file
    help      Prints this message or the help of the given subcommand(s)
```
Huffman features 2 modes, encoding and decoding. The typical use case is first using the `encode`
command to compress a file, and then the `decode` file to decompress it later.

## Encoding
```
USAGE:
    huffman encode <input> -o <output>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o <output>        The output file to put the decoded text into

ARGS:
    <input>    The input file to encode
```
This encodes a file by counting the occurrences of each byte in the file,
and using that to construct a Huffman tree and assign a bit pattern to each byte.
The output is a binary file, prefixed with the byte counts, and then followed
by a stream of encoded bytes. Because we include the byte counts at the start of the file,
we can rebuild the Huffman tree when decompressing the file.

## Decoding
```
USAGE:
    huffman decode <input> -o <output>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o <output>        The output file to put the decoded text into

ARGS:
    <input>    The input file to decode
```
This is the reverse of the encoding operation. This must be used on a file
encoded with the same version of the program, otherwise unkown results will happen.
