import { atom } from "recoil";
import { createContextState } from "foxact/create-context-state";

const [ThemeModeProvider, useThemeMode, useSetThemeMode] = createContextState<
  "light" | "dark"
>("light");

const [LogDataProvider, useLogData, useSetLogData] = createContextState<
  ILogItem[]
>([]);

export const atomEnableLog = atom<boolean>({
  key: "atomEnableLog",
  effects: [
    ({ setSelf, onSet }) => {
      const key = "enable-log";

      try {
        setSelf(localStorage.getItem(key) !== "false");
      } catch {}

      onSet((newValue, _, isReset) => {
        try {
          if (isReset) {
            localStorage.removeItem(key);
          } else {
            localStorage.setItem(key, newValue.toString());
          }
        } catch {}
      });
    },
  ],
});

interface IConnectionSetting {
  layout: "table" | "list";
}

export const atomConnectionSetting = atom<IConnectionSetting>({
  key: "atomConnectionSetting",
  effects: [
    ({ setSelf, onSet }) => {
      const key = "connections-setting";

      try {
        const value = localStorage.getItem(key);
        const data = value == null ? { layout: "table" } : JSON.parse(value);
        setSelf(data);
      } catch {
        setSelf({ layout: "table" });
      }

      onSet((newValue) => {
        try {
          localStorage.setItem(key, JSON.stringify(newValue));
        } catch {}
      });
    },
  ],
});

// save the state of each profile item loading
const [LoadingCacheProvider, useLoadingCache, useSetLoadingCache] =
  createContextState<Record<string, boolean>>({});

// save update state
const [UpdateStateProvider, useUpdateState, useSetUpdateState] =
  createContextState<boolean>(false);

export {
  ThemeModeProvider,
  useThemeMode,
  useSetThemeMode,
  LogDataProvider,
  useLogData,
  useSetLogData,
  LoadingCacheProvider,
  useLoadingCache,
  useSetLoadingCache,
  UpdateStateProvider,
  useUpdateState,
  useSetUpdateState,
};
