from ultralytics import YOLO

yolo_26, yolo_11 = YOLO("yolo26n.pt"), YOLO("yolo11n.pt")
yolo_26.export(format="onnx", imgsz=640,project="../models/")
print("done exporting yolo 26")
yolo_11.export(format="onnx", imgsz=640, project="../models/")
print("done exporting yolo 26")
results = model("path/to/bus.jpg")  # run inference
println(f"The yolo26 RESULTS:\n ${results}");
