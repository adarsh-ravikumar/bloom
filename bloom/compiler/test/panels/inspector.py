from core import BloomPanel, BloomState
from app import Global

from generated import InspectorPanel, ExportOptions, Format, BloomButton, BloomSlider, SliderChange, ButtonClick, MouseButton


class Inspector(InspectorPanel):
    def __init__(self):
        print("Inspector panel opened")

        self.brightness.on_delta(
            lambda value: Global.image_engine.set_brightness(value))

        self.contrast.on_delta(
            lambda value: Global.image_engine.set_contrast(value))

        self.saturation.on_delta(
            lambda value: Global.image_engine.set_saturation(value))

        self.hue.on_delta(
            lambda value: Global.image_engine.set_hue(value))

        super()

    def on_export_image(self, event: ExportOptions):
        format = ""
        match event.format:
            case Format.PNG:
                format = "png"
            case Format.JPG:
                format = "jpg"
            case Format.SVG:
                format = "svg"

        Global.image_engine.export(format, event.scale)

    def on_cw_click(self, event: ButtonClick):
        if event.button == MouseButton.left:
            Global.image_engine.rotate_cw()

    def on_ccw_click(self, event: ButtonClick):
        if event.button == MouseButton.left:
            Global.image_engine.rotate_ccw()


bloom.register("inspector", Inspector)
