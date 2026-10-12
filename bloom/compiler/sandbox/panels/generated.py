from core import BloomPanel, BloomState
from app import Global


# core/widget/slider
class BloomSlider:
    pass


class SliderChange:
    value: float


# core/widget/button
class BloomButton:
    pass


class MouseButton:
    left = 0
    right = 1
    middle = 2


class ButtonClick:
    double: bool
    button: MouseButton
    mouse_x: float
    mouse_y: float


# panel/inspector

class Format:
    PNG = 0
    JPG = 0
    SVG = 0


class ExportOptions:
    format: Format
    scale: float


class InspectorPanel(BloomPanel):
    brightness: BloomState[float]
    contrast: BloomState[float]
    saturation: BloomState[float]
    hue: BloomState[float]

    w_slider_brightness: BloomSlider
    w_slider_contrast: BloomSlider
    w_slider_saturation: BloomSlider
    w_slider_hue: BloomSlider
    w_button_cw: BloomButton
    w_button_ccw: BloomButton

    def __init__(self):
        super()

    def on_export_image(self, event: ExportOptions):
        pass

    def on_cw_click(self, event: ButtonClick):
        pass

    def on_ccw_click(self, event: ButtonClick):
        pass
