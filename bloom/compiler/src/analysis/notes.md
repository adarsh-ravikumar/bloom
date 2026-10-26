There are three types of `.bloom` files.

1) Widgets
2) Panels
3) Layouts

# Widgets

Widgets are the building blocks of a bloom application. They compose primitive and custom widgets to represent a particular state.
A state is the fundamental unit of reactivity of a bloom application. The host may provide the states with values at any point of the lifecycle.

Widgets emit events and expose commands. Events notify listeners of change. Command executes some functionality on the widget.

# Panels

Panels are composed of widgets. Like widgets, they also have states, events, and commands. But unlike widgets, panels themselves cannot be composed within other panels or widgets. One might, however, compose panels in layouts.

# Layouts

Layouts are ways in which the author can specify visual compositions for panels, like tabs, splits, etc. Unlike panels and widgets, layouts do NOT allow for any reactivity. It exists purely as a way to, well, lay-out panels and widgets in semantically sensible ways.
