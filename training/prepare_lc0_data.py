"""TODO: Proper docs.

lc0 training data can be downloaded from
https://storage.lczero.org/files/training_data/
"""

import ctypes
import os
import sys
import tarfile
from typing import List
import zlib
import chess
import idx_to_move
import numpy as np
import tqdm

# This corresponds to 900 CP or +-9 "pawns".
Q_THRESHOLD: float = 0.9
PIECES_THRESHOLD = 6
BOARD_SIZE = 64
NUM_PLANES = 12


class V6TrainingData(ctypes.LittleEndianStructure):
    """Latest training data format from lc0.

    https://lczero.org/dev/wiki/training-data-format-versions/
    """

    # Make sure the
    _pack_ = 1
    _fields_ = [
        ("version", ctypes.c_uint32),
        ("input_format", ctypes.c_uint32),
        ("probabilities", ctypes.c_float * 1858),
        ("planes", ctypes.c_uint64 * 104),
        ("castling_us_ooo", ctypes.c_uint8),
        ("castling_us_oo", ctypes.c_uint8),
        ("castling_them_ooo", ctypes.c_uint8),
        ("castling_them_oo", ctypes.c_uint8),
        ("side_to_move_or_enpassant", ctypes.c_uint8),
        ("rule50_count", ctypes.c_uint8),
        ("invariance_info", ctypes.c_uint8),
        ("dummy", ctypes.c_uint8),
        ("root_q", ctypes.c_float),
        ("best_q", ctypes.c_float),
        ("root_d", ctypes.c_float),
        ("best_d", ctypes.c_float),
        ("root_m", ctypes.c_float),
        ("best_m", ctypes.c_float),
        ("plies_left", ctypes.c_float),
        ("result_q", ctypes.c_float),
        ("result_d", ctypes.c_float),
        ("played_q", ctypes.c_float),
        ("played_d", ctypes.c_float),
        ("played_m", ctypes.c_float),
        ("orig_q", ctypes.c_float),
        ("orig_d", ctypes.c_float),
        ("orig_m", ctypes.c_float),
        ("visits", ctypes.c_uint32),
        ("played_idx", ctypes.c_uint16),
        ("best_idx", ctypes.c_uint16),
        ("policy_kld", ctypes.c_float),
        ("reserved", ctypes.c_uint32),
    ]


assert ctypes.sizeof(V6TrainingData) == 8356, "Wrong struct size"


def extract_training_samples(archive_path: str) -> List[V6TrainingData]:
    archive = tarfile.open(archive_path, "r:*")

    samples: List[V6TrainingData] = list()

    for member in archive.getmembers():
        if not member.isfile() or not member.name.endswith(".gz"):
            continue
        data = archive.extractfile(member)
        if data is None:
            continue

        # This file contains a sequence of serialized V6TrainingData structs.
        file_contents = zlib.decompress(data.read(), 15 + 16)
        assert (
            len(file_contents) % ctypes.sizeof(V6TrainingData) == 0
        ), "File size not multiple of struct size"

        for i in range(len(file_contents) // ctypes.sizeof(V6TrainingData)):
            serialized_struct_bytes = file_contents[
                i * ctypes.sizeof(V6TrainingData) : (i + 1)
                * ctypes.sizeof(V6TrainingData)
            ]
            if not serialized_struct_bytes:
                break
            next_sample = V6TrainingData()
            ctypes.memmove(
                ctypes.addressof(next_sample),
                serialized_struct_bytes,
                ctypes.sizeof(V6TrainingData),
            )

            samples.append(next_sample)

    return samples


# lc0 mirros the bits in each byte for some reason...
# But the moves are in the correct form.
def mirror_horizontally(x: np.uint64):
    k1 = np.uint64(0x5555555555555555)
    k2 = np.uint64(0x3333333333333333)
    k4 = np.uint64(0x0F0F0F0F0F0F0F0F)
    x = ((x >> np.uint64(1)) & k1) | ((x & k1) << np.uint64(1))
    x = ((x >> np.uint64(2)) & k2) | ((x & k2) << np.uint64(2))
    x = ((x >> np.uint64(4)) & k4) | ((x & k4) << np.uint64(4))
    return x


def extract_features(sample: V6TrainingData) -> np.array:
    features = np.array(sample.planes[0:NUM_PLANES], dtype=np.uint64)
    for i in range(NUM_PLANES):
        features[i] = mirror_horizontally(features[i])
    return features


def should_filter(sample: V6TrainingData) -> bool:
    # Drop samples that we can't process yet.
    if sample.version != 6 or sample.input_format != 1:
        return True
    # Drop samples maarked for deletion by rescorer.
    if sample.invariance_info & (1 << 6):
        return True
    if sample.side_to_move_or_enpassant > 1:
        return True

    # IMPORTANT: Drop samples with high q (likely tactical position)
    if abs(sample.best_q) > Q_THRESHOLD:
        return True

    # IMPORTANT: Drop samples with less than a certain number of pieces
    # because it is either a tablebase position or very close to it.
    features = extract_features(sample)
    if np.unpackbits(features.view(np.uint8)).sum() <= PIECES_THRESHOLD:
        return True

    # Advanced checks: shouldn't be in check or stalemate.
    # The next move also shouldn't be a capture or give a check.
    board = chess.Board.empty()
    plane_id = 0

    for color in (chess.WHITE, chess.BLACK):
        for piece in (
            chess.PAWN,
            chess.KNIGHT,
            chess.BISHOP,
            chess.ROOK,
            chess.QUEEN,
            chess.KING,
        ):
            plane = features[plane_id]
            for square in range(BOARD_SIZE):
                if int(plane) & (1 << square):
                    board.set_piece_at(square, chess.Piece(piece, color))
            plane_id += 1

    if board.is_check():
        return True
    if board.is_stalemate():
        return True

    move = chess.Move.from_uci(idx_to_move.IDX_TO_MOVE[sample.best_idx])
    # This will include en_passants
    # TODO: Maybe allow capturing moves? Even though it's tactics, it should
    # still be useful.
    if board.is_capture(move):
        return True
    # if board.gives_check(move):
    #   return True
    # if board.is_castling(move):
    #   return True

    return False


def process_archive(archive_path: str, features_path, targets_path) -> int:
    print(f"Processing {archive_path}")

    extracted = 0
    dropped = 0

    samples = extract_training_samples(archive_path)

    features = []
    targets = []

    for sample in tqdm.tqdm(samples):
        if should_filter(sample):
            dropped += 1
            continue
        features.append(extract_features(sample))
        targets.append(sample.best_q)
        extracted += 1

    features = np.array(features)
    targets = np.array(targets, dtype=np.float32)

    assert features.shape == (extracted, NUM_PLANES)
    assert targets.shape == (extracted,)

    # Remove duplicate positions and their evaluations.
    _, unique_idx = np.unique(features, return_index=True, axis=0)
    features = features[unique_idx]
    targets = targets[unique_idx]

    num_samples = len(unique_idx)

    print(f"Extracted {extracted:_} samples, dropped {dropped:_}")
    print(f"num_samples after deduplication: {num_samples:_}")

    features.tofile(features_path)
    targets.tofile(targets_path)

    return num_samples


def process_archives(archives: List[str], features_path, targets_path):
    samples = 0
    bar = tqdm.tqdm(archives)
    for archive_path in bar:
        sampls += process_archive(archive_path, features_path, targets_path)
        bar.set_description(f"extracted {samples} samples")
    print(f"Extracted total {samples:_} samples")


if __name__ == "__main__":
    # Read from the input file and save to output_dir with a different extension.
    input_filename = sys.argv[1]
    output_dir = "~/Development/data/processed/"

    filename = os.path.basename(input_filename)
    # Remove the file extension and change it to .data
    features_filename = os.path.splitext(filename)[0] + "_features.data"
    targets_filename = os.path.splitext(filename)[0] + "_targets.data"
    # Create the new file path in the output directory
    features_path = os.path.join(output_dir, features_filename)
    targets_path = os.path.join(output_dir, targets_filename)

    if os.path.exists(features_path) or os.path.exists(targets_path):
        print(f"The data file for {input_filename} already exists, will not run again")
        sys.exit()

    print(f"Reading from {input_filename}")
    print(f"Will write features to {features_path}")
    print(f"Will write targets to {targets_path}")

    process_archive(sys.argv[1], features_path, targets_path)
