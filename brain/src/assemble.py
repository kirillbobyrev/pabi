import glob
import numpy as np
import tqdm

all_features = None
all_targets = None

target_files = sorted(glob.glob("*targets.data"))
features_files = sorted(glob.glob("*features.data"))

assert len(target_files) == len(features_files)

bar = tqdm.tqdm(zip(features_files, target_files), total=len(target_files))
for feature_file, target_file in bar:
    new_features = np.fromfile(feature_file, dtype=np.uint64)
    new_targets = np.fromfile(target_file, dtype=np.float32)

    num_samples = new_targets.shape[0]
    assert new_features.shape[0] % 12 == 0
    assert new_features.shape[0] // 12 == num_samples

    new_features = new_features.reshape((num_samples, 12))

    if all_features is None and all_targets is None:
        all_features = new_features
        all_targets = new_targets
        continue

    all_features = np.concatenate((all_features, new_features), axis=0)
    all_targets = np.concatenate((all_targets, new_targets), axis=0)

    _, idx = np.unique(all_features, axis=0, return_index=True)

    all_features = all_features[idx]
    all_targets = all_targets[idx]

    bar.set_description(f"Total {all_features.shape[0]:_} samples")


all_features.tofile("features.data")
all_targets.tofile("targets.data")
