import math
import os
from jax import numpy as jnp
import numpy as np
import tqdm


FEATURES_SIZE = 12 * 8
TARGET_SIZE = 4


class DataLoader:
    TRAIN_RATIO: float = 0.94
    DEV_RATIO: float = 0.04
    TEST_RATIO: float = 0.02

    def __init__(self, features_path, targets_path, batch_size):
        self.batch_size = batch_size

        self.features_file = open(features_path, "rb")
        self.targets_file = open(targets_path, "rb")

        assert os.stat(features_path).st_size % FEATURES_SIZE == 0
        assert os.stat(targets_path).st_size % TARGET_SIZE == 0
        assert (
            os.stat(features_path).st_size // FEATURES_SIZE
            == os.stat(targets_path).st_size // TARGET_SIZE
        )

        self.num_samples = os.stat(features_path).st_size // FEATURES_SIZE

        print(f"Initialized DataLoader and with {self.num_samples} samples")

        self.num_train_samples = int(self.num_samples * self.TRAIN_RATIO)
        print(f"Train samples: {self.num_train_samples}")
        self.num_dev_samples = int(self.num_samples * self.DEV_RATIO)
        print(f"Dev samples: {self.num_dev_samples}")
        self.num_test_samples = (
            self.num_samples - self.num_train_samples - self.num_dev_samples
        )
        print(f"Test samples: {self.num_test_samples}")

        assert (
            self.num_train_samples
            + self.num_dev_samples
            + self.num_test_samples
            == self.num_samples
        )

    def _iter(self, start_idx, end_idx, shuffle):
        batch_count = math.ceil((end_idx - start_idx) / self.batch_size)
        # print(f"Batch count: {batch_count}")
        batch_indices = np.arange(batch_count)
        # print(f"Batch indices: {batch_indices}")
        if shuffle:
            np.random.shuffle(batch_indices)
        for batch_idx in tqdm.tqdm(batch_indices):
            batch_start_idx = start_idx + batch_idx * self.batch_size
            batch_end_idx = min(
                end_idx, start_idx + (batch_idx + 1) * self.batch_size
            )
            effective_batch_size = batch_end_idx - batch_start_idx

            self.features_file.seek(batch_start_idx * FEATURES_SIZE)
            self.targets_file.seek(batch_start_idx * TARGET_SIZE)

            # print(f"batch range: [{batch_start_idx}, {batch_end_idx})")
            features = (
                np.unpackbits(
                    np.fromfile(
                        self.features_file,
                        dtype=np.uint8,
                        count=effective_batch_size * FEATURES_SIZE,
                    ),
                    bitorder="little",
                )
                .reshape(effective_batch_size, -1)
                .astype(np.float32)
            )
            self.features_file.seek(batch_start_idx * FEATURES_SIZE)
            targets = (
                np.fromfile(
                    self.targets_file,
                    dtype=np.float32,
                    count=effective_batch_size,
                )
                .reshape(effective_batch_size, 1)
                .astype(np.float32)
                / 100
            )
            # print(f"Read features of shape {features.shape}")
            # print(f"Read targets of shape {targets.shape}")
            yield (
                jnp.array(features, dtype=jnp.float32),
                jnp.array(targets, dtype=jnp.float32),
            )

    def train_ds(self, shuffle=True):
        return self._iter(0, self.num_train_samples, shuffle=shuffle)

    def dev_ds(self):
        return self._iter(
            self.num_train_samples,
            self.num_train_samples + self.num_dev_samples,
            shuffle=False,
        )

    def test_ds(self):
        return self._iter(
            self.num_train_samples + self.num_dev_samples,
            self.num_samples,
            shuffle=False,
        )


if __name__ == "__main__":
    loader = DataLoader(
        "./data/processed/features.data",
        "./data/processed/targets.data",
        batch_size=10,
    )

    features, targets = next(loader.dev_ds(), shuffle=False)
    print(features)
    print(targets)
