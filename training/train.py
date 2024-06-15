import os

os.environ["KERAS_BACKEND"] = "jax"

import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns

from tensorflow import keras
from tensorflow.keras import layers


class BatchLossHistory(keras.callbacks.Callback):
  history = []

  def on_train_batch_end(self, batch, logs=None):
    self.history.append(logs["loss"])

  def on_epoch_end(self, epoch, logs=None):
    plt.plot(self.history)
    plt.ylim(top=logs["val_loss"] * 2, bottom=0)
    plt.ylabel("Training loss")
    plt.xlabel("Training batch")
    plt.legend()
    plt.show()


batch_loss_history = BatchLossHistory()

checkpoint_filepath = "checkpoint.model.keras"
checkpoint_callback = keras.callbacks.ModelCheckpoint(
  filepath=checkpoint_filepath,
  monitor="val_loss",
  mode="auto",
  save_best_only=True,
)

reduce_lr = keras.callbacks.ReduceLROnPlateau(
  monitor="val_loss", factor=0.1, patience=2, min_lr=1e-5
)

early_stopping = keras.callbacks.EarlyStopping(monitor="val_loss", patience=5)

input_dim = 768

model = keras.Sequential(
  [
    layers.Dense(512, input_dim=input_dim, activation="silu"),
    layers.Dense(256, input_dim=512, activation="silu"),
    layers.Dense(1, input_dim=256, activation="sigmoid"),
    # Rescale the output to [-10, 10] for numerical stability.
    layers.Rescaling(scale=10 * 2, offset=-10),
  ]
)

optimizer = keras.optimizers.AdamW(learning_rate=LEARNING_RATE)
model.compile(optimizer=optimizer, loss="mse")

model.summary()

model.fit(
  train,
  epochs=NUM_EPOCHS,
  steps_per_epoch=STEPS_PER_EPOCH,
  validation_data=val,
  validation_steps=STEPS_PER_VAL_EPOCH,
  callbacks=[
    reduce_lr,
    batch_loss_history,
    checkpoint_callback,
    early_stopping,
  ],
)

plt.plot(batch_loss_history.history, label="batch training loss")
plt.xlabel("Training step")
plt.ylim(bottom=0)
plt.legend()
plt.show()
