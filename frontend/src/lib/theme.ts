// Simple theme management with localStorage persistence.

type Theme = "light" | "dark";

function getStored(): Theme {
  if (typeof window === "undefined") return "light";
  return (localStorage.getItem("epeletii-theme") as Theme) || "light";
}

function apply(theme: Theme) {
  if (typeof window === "undefined") return;
  document.documentElement.classList.toggle("dark", theme === "dark");
  localStorage.setItem("epeletii-theme", theme);
}

let _current: Theme = "light";

export const theme = {
  get current() { return _current; },
  init() {
    _current = getStored();
    apply(_current);
  },
  toggle() {
    _current = _current === "light" ? "dark" : "light";
    apply(_current);
  },
};
