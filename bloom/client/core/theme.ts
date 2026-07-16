import "@style/global.scss";

import "@style/themes/dark.scss";
import "@style/themes/light.scss";

export const Themes = Object.freeze({
  dark: "theme-dark",
  light: "theme-light"
});



export class ThemeManager {
  private _current!: string;

  constructor(theme: string) {
    this.theme = theme;
  }

  public get theme(): string {
    return this._current;
  }

  public set theme(theme: string) {
    this._current = theme;

    Object.values(Themes).forEach((val) => {
      document.documentElement.classList.remove(val);
    })

    document.documentElement.classList.add(this._current);
  }
}
