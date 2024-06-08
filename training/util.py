import numpy as np


def mirror_horizontally(x: np.uint64):
    k1 = np.uint64(0x5555555555555555)
    k2 = np.uint64(0x3333333333333333)
    k4 = np.uint64(0x0F0F0F0F0F0F0F0F)
    x = ((x >> np.uint64(1)) & k1) | ((x & k1) << np.uint64(1))
    x = ((x >> np.uint64(2)) & k2) | ((x & k2) << np.uint64(2))
    x = ((x >> np.uint64(4)) & k4) | ((x & k4) << np.uint64(4))
    return x


# Useful for debugging.
def print_bitboard(bitboard: np.uint64):
    # Convert the bitboard to a 64-bit binary string.
    binary_string = f"{bitboard:064b}"

    # Print by chunks of 8 squares on each line.
    for i in range(0, 64, 8):
        print(binary_string[i : i + 8])
