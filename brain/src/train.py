import os
import time
from flax import linen as nn
from flax.linen import initializers
from flax.training import checkpoints
from flax.training import train_state
import jax
from jax import numpy as jnp
from jax import random
import matplotlib.pyplot as plt
import numpy as np
import optax
import gc
import tqdm
import math
import seaborn as sns


targets_sample = np.fromfile("targets.data", dtype=np.float32, count=100_000)
print(
    f"Targets shape = {targets_sample.shape[0]:_}, "
    f"mean = {targets_sample.mean():.5f}, "
    f"std = {targets.std():.5f}"
)

fig = sns.histplot(targets_sample)
fig.set_yscale("log")
fig.set_xlabel("Target (q-value prediction from lc0)")
fig.show()

NUM_EPOCHS = 30
BATCH_SIZE = 200_000
LEARNING_RATE = 5e-5

FEATURES_SIZE = 12 * 8
TARGET_SIZE = 4


class DataLoader:
    TRAIN_PERCENTAGE: float = 0.94
    DEV_PERCENTAGE: float = 0.04
    TEST_PERCENTAGE: float = 0.02

    def __init__(self, features_path, targets_path, batch_size):
        assert self.TRAIN_PERCENTAGE > 0
        assert self.DEV_PERCENTAGE > 0
        assert self.TEST_PERCENTAGE > 0
        assert (
            self.TRAIN_PERCENTAGE + self.DEV_PERCENTAGE + self.TEST_PERCENTAGE
            == 100
        )

        self.batch_size = batch_size

        self.features_file = open(features_path, "rb")
        self.targets_file = open(targets_path, "rb")

        self.num_samples = os.stat(targets_path).st_size // TARGET_SIZE

        assert os.stat(features_path).st_size % FEATURES_SIZE == 0
        assert (
            os.stat(features_path).st_size // FEATURES_SIZE == self.num_samples
        )
        assert os.stat(targets_path).st_size % TARGET_SIZE == 0

        print(f"Initialized DataLoader and with {self.num_samples:_} samples")

        self.num_train_samples = int(
            self.num_samples * self.TRAIN_PERCENTAGE / 100
        )
        self.num_dev_samples = int(
            self.num_samples * self.DEV_PERCENTAGE / 100
        )
        self.num_test_samples = (
            self.num_samples - self.num_train_samples - self.num_dev_samples
        )

        print(f"Train samples: {self.num_train_samples:_}")
        print(f"Dev samples: {self.num_dev_samples:_}")
        print(f"Test samples: {self.num_test_samples:_}")

        assert (
            self.num_train_samples
            + self.num_dev_samples
            + self.num_test_samples
            == self.num_samples
        )

    def _iter(self, start_idx, end_idx, shuffle):
        batch_count = math.ceil((end_idx - start_idx) / self.batch_size)
        batch_indices = np.arange(batch_count)
        if shuffle:
            np.random.shuffle(batch_indices)
        for batch_idx in batch_indices:
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
            # Divide by 100 (?)
            targets = (
                np.fromfile(
                    self.targets_file,
                    dtype=np.float32,
                    count=effective_batch_size,
                )
                .reshape(effective_batch_size, 1)
                .astype(np.float32)
            )
            # print(f"Read features of shape {features.shape}")
            # print(f"Read targets of shape {targets.shape}")
            yield (
                jnp.array(features, dtype=jnp.float32),
                jax.nn.sigmoid(jnp.array(targets, dtype=jnp.float32)),
            )

    def train_ds(self):
        return self._iter(0, self.num_train_samples, shuffle=True)

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


loader = DataLoader(
    "features.data",
    "targets.data",
    BATCH_SIZE,
)


class BaselineMLP(nn.Module):
    """Basic MLP model similar to one Stockfish started from."""

    NUM_PREDICTIONS: int = 1

    @nn.compact
    def __call__(self, x, train: bool, rng: jax.random.PRNGKey):
        x = nn.Dense(features=512)(x)
        x = nn.relu(x)
        x = nn.Dense(features=self.NUM_PREDICTIONS)(x)
        x = nn.sigmoid(x) * 2 - 1
        return x


class PabiBrain(nn.Module):
    NUM_PREDICTIONS: int = 1

    # Model hyperparameters.
    dropout_rate: float = 0.25
    use_dropout: bool = False

    @nn.compact
    def __call__(self, x: jnp.array, train: bool, rng: jax.random.PRNGKey):
        x = nn.Dense(features=512, kernel_init=initializers.he_normal())(x)
        x = nn.silu(x)
        if self.use_dropout:
            x = nn.Dropout(rate=self.dropout_rate)(
                x, deterministic=not train, rng=rng
            )
        x = nn.Dense(features=128, kernel_init=initializers.he_normal())(x)
        x = nn.silu(x)
        if self.use_dropout:
            x = nn.Dropout(rate=self.dropout_rate)(
                x, deterministic=not train, rng=rng
            )
        x = nn.Dense(features=32, kernel_init=initializers.he_normal())(x)
        x = nn.silu(x)
        x = nn.Dense(
            features=self.NUM_PREDICTIONS,
            kernel_init=nn.initializers.xavier_normal(),
        )(x)
        x = nn.sigmoid(x) * 2 - 1
        return x


INPUT_FEATURES = 768


def create_train_state(
    module: nn.Module, rng: jax.random.PRNGKey, learning_rate: float
):
    params = module.init(
        rng,
        jnp.ones([1, INPUT_FEATURES], dtype=np.float32),
        train=False,
        rng=rng,
    )["params"]
    tx = optax.adam(learning_rate)
    return train_state.TrainState.create(
        apply_fn=module.apply, params=params, tx=tx
    )


@jax.jit
def train_step(
    state: train_state.TrainState, batch: jnp.array, rng: jax.random.PRNGKey
):
    """Train for a single step."""

    def loss_fn(params):
        (features, targets) = batch
        prediction = state.apply_fn(
            {"params": params}, features, train=True, rng=rng
        )
        # MSE.
        loss = optax.squared_error(prediction, targets).mean()

        return loss

    grad_fn = jax.value_and_grad(loss_fn)
    loss, grads = grad_fn(state.params)
    state = state.apply_gradients(grads=grads)
    return state, loss


def evaluate_model(
    state: train_state.TrainState,
    ds,
    rng: jax.random.PRNGKey,
):
    losses = []
    for batch in ds():
        positions, targets = batch
        prediction = state.apply_fn(
            {"params": state.params}, positions, train=False, rng=rng
        )
        # MSE.
        loss = optax.squared_error(prediction, targets).mean()
        losses.append(loss)
    return np.array(losses).mean()


def get_learning_rate(epoch, initial_lr):
    if epoch < 5:
        return initial_lr
    if epoch < 10:
        return initial_lr * 0.5
    elif epoch < 15:
        return initial_lr * 0.2
    else:
        return initial_lr * 0.05


def plot_metrics(metrics):
    # Plotting the errors
    plt.figure(figsize=(10, 5))
    plt.plot(metrics["train_loss"], label="Training Loss")
    plt.plot(metrics["dev_loss"], label="Validation Loss")
    plt.plot(metrics["test_loss"], label="Test Loss")
    plt.xlabel("Epochs")
    plt.ylabel("Loss")
    plt.title("Training and Validation Loss Over Epochs")
    plt.legend()

    plt.ylim(bottom=0)

    # Save the plot to disk
    plot_path = os.path.abspath("loss_plot.png")
    plt.savefig(plot_path)
    plt.show()
    print(f"Loss plot saved to {plot_path}")


rng = random.PRNGKey(0)
module = PabiBrain(use_dropout=True)

state = create_train_state(module, rng, LEARNING_RATE)

checkpoint_dir = os.path.abspath("checkpoints")
os.makedirs(checkpoint_dir, exist_ok=True)

best_dev_loss = evaluate_model(state, loader.dev_ds, rng)
test_loss = evaluate_model(state, loader.test_ds, rng)

print(
    f"Starting train loss: {train_loss:.5f}, dev loss: {best_dev_loss:.5f},"
    f" test loss: {test_loss:.5f}"
)

metrics_history = {
    "train_loss": [],
    "dev_loss": [],
    "test_loss": [],
}

for epoch in range(1, NUM_EPOCHS + 1):
    start_time = time.time()

    # Update the learning rate based on the epoch
    current_lr = get_learning_rate(epoch, LEARNING_RATE)
    tx = optax.adam(current_lr)
    state = state.replace(tx=tx)

    train_losses = []
    epoch_rng, rng = random.split(rng)

    bar = tqdm.tqdm(loader.train_ds())
    for batch in bar:
        batch_rng, epoch_rng = random.split(epoch_rng)
        state, loss = train_step(state, batch, batch_rng)
        bar.set_description(f"batch loss: {loss:.5f}")
        train_losses.append(loss)
    train_loss = np.array(train_losses).mean()
    dev_loss = evaluate_model(state, loader.dev_ds, epoch_rng)

    epoch_time = time.time() - start_time  # Calculate epoch duration

    print(
        f"\nEpoch {epoch} train loss = {train_loss:.5f} dev loss:"
        f" {dev_loss:.5f} time = {epoch_time / 60:.2f} minutes, "
        f"current learning rate = {current_lr:.5f}"
    )
    metrics_history["train_loss"].append(train_loss)
    metrics_history["dev_loss"].append(dev_loss)
    metrics_history["test_loss"].append(test_loss)

    if dev_loss < best_dev_loss:
        best_dev_loss = dev_loss
        test_loss = evaluate_model(state, loader.test_ds, epoch_rng)
        print(
            f"Updating best checkpoint: dev loss = {best_dev_loss:.5f}, "
            f"test loss = {test_loss:.5f}"
        )
        checkpoints.save_checkpoint(
            ckpt_dir=checkpoint_dir,
            target=state,
            step=epoch,
            prefix="best_",
            overwrite=True,
        )

    if epoch > 0 and epoch % 10 == 0:
        plot_metrics(metrics_history)

    gc.collect()

print(
    f"Final best dev loss = {best_dev_loss:.5f}, "
    f"test loss = {test_loss:.5f}"
)
