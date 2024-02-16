## Assembly Control Flow Graph Generator

This program generates a control flow graph, based on a given binary and a virtual address.

### Pipe it into an output file, there are debug print outs during the creation process

rm output.txt &&

cargo run -- test_bins/vec_iter 0x00405fe9  >> output.txt && dot -Tpng output.dot -o example.png

cargo run -- test_bins/output_executable 0x00405fe5  >> output.txt && dot -Tpng output.dot -o example.png