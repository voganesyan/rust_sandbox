import tensorflow as tf

# default input shape 224x224x3
model = tf.keras.applications.MobileNetV3Small(
    input_shape=(224, 224, 3), weights="imagenet"
)

# save the model
directory = "mobilenetv3"
model.save(directory, save_format="tf")

# load sample image
buf = tf.io.read_file("data/macaque.png")
img = tf.image.decode_png(buf)

# check model prediction
predict = model(img[tf.newaxis, :, :, :])
predict = predict.numpy()
decoded = tf.keras.applications.imagenet_utils.decode_predictions(predict, top=1)[0]

print(f"""argmax={predict.argmax(axis=1)[0]}""")
print("class_name | class_description | score")
print("-----------+-------------------+------")
print(f"{decoded[0][0]:>10} | {decoded[0][1]:>17} | {decoded[0][2]:0.3f}")
