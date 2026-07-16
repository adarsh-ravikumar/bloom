import type { App } from "@runtime/app";
import type { GenericWidget } from "@core/widget";
import type { Props } from "@core/props";
import { Computed, type State } from "@core/state";

export interface BloomSession {
  version: number;
  appName: string;
  workspace: SerializedWidget;
}

export interface SerializedWidget {
  widgetID: number;
  constructorID: number;

  props: SerializedProps;

  html: string;

  children: SerializedWidget[];
}

export type SerializedProps = Record<string, unknown>;

export interface SerializedState {
  value: any,
  type: "computed" | "state"
}

function serializeState(state: State<any>): unknown {
  if (state instanceof Computed) {
    return {
      type: "computed",
      value: state.value,
    };
  }

  return {
    type: "state",
    value: state.value,
  };
}

function serializeProps(props: Props<any>): SerializedProps {
  const serialized: SerializedProps = {};

  for (const [key, state] of props.getStates()) {
    serialized[key] = serializeState(state);
  }

  return serialized;
}

function serializeWidget(widget: GenericWidget): SerializedWidget {
  return {
    widgetID: widget.id,
    constructorID: widget.constructorID,
    props: serializeProps(widget.props),
    html: widget.html,
    children: widget.children.map((child) => serializeWidget(child))
  }
}

export function serialize(app: App): BloomSession {
  return {
    version: 1,
    appName: app.appName,
    workspace: serializeWidget(app.workspace)
  }
}
