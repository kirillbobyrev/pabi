import glob
import numpy as np
import tqdm

FILE_LIMIT = 200

all_features = None
all_targets = None

target_files = sorted(glob.glob("*targets.data")[:FILE_LIMIT])
features_files = sorted(glob.glob("*features.data")[:FILE_LIMIT])

assert len(target_files) == len(features_files)

print("Reading all files")
all_features = np.concatenate([np.fromfile(f, dtype=np.uint64) for f in features_files])
all_targets = np.concatenate([np.fromfile(f, dtype=np.float32) for f in target_files])

num_samples = all_targets.shape[0]

assert all_features.shape[0] % 12 == 0
assert all_features.shape[0] // 12 == num_samples

all_features = all_features.reshape(num_samples, 12)

assert all_features.shape[0] == all_targets.shape[0]

print(f"Deduplicating {num_samples:_} samples")
_, idx = np.unique(all_features, axis=0, return_index=True)

print(f"After deduplication: {len(idx):_} samples left")
all_features = all_features[idx]
all_targets = all_targets[idx]

all_features.tofile("features.data")
all_targets.tofile("targets.data")
