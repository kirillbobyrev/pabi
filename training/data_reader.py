import numpy as np
import os

NUM_FEATURES = 768
NUM_PLANES = 12


def data_generator(features_file_path, scores_file_path, batch_size):
  features_file = open(features_file_path, "rb")
  scores_file = open(scores_file_path, "rb")

  assert os.stat(features_file_path).st_size % (NUM_PLANES * 8) == 0
  assert os.stat(scores_file_path).st_size % 2 == 0
  assert (
    os.stat(features_file_path).st_size // (NUM_PLANES * 8)
    == os.stat(scores_file_path).st_size // 2
  )

  num_samples = os.stat(scores_file_path).st_size // 2

  while True:
    batch_indices = np.arange(num_samples // batch_size)
    np.random.shuffle(batch_indices)

    for batch_idx in batch_indices:
      batch_start = batch_idx * batch_size
      batch_end = min(batch_start + batch_size, num_samples)
      effective_batch_size = batch_end - batch_start

      features_file.seek(batch_start * NUM_PLANES * 8)
      scores_file.seek(batch_start * 2)

      features = np.fromfile(
        features_file,
        dtype=np.uint64,
        count=effective_batch_size * NUM_PLANES,
      )
      features = np.unpackbits(
        features.view(np.uint8), bitorder="little"
      ).astype(np.float32)
      features = features.reshape(-1, NUM_FEATURES)

      scores = np.fromfile(
        scores_file, dtype=np.int16, count=effective_batch_size
      )
      scores = scores.reshape(-1, 1).astype(np.float32) / 100

      yield features, scores
