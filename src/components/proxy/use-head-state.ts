import { useCallback, useEffect, useState } from "react";
import { ProxySortType } from "./use-filter-sort";
import { useProfiles } from "@/hooks/use-profiles";

export interface HeadState {
  open?: boolean;
  showType: boolean;
  sortType: ProxySortType;
  filterText: string;
  textState: "url" | "filter" | null;
  testUrl: string;
  // 隐藏无效网络
  hiddenInvalidNetwork: boolean;
}

type HeadStateStorage = Record<string, Record<string, HeadState>>;

const HEAD_STATE_KEY = "proxy-head-state";
export const DEFAULT_STATE: HeadState = {
  open: false,
  showType: true,
  sortType: 0,
  filterText: "",
  textState: null,
  testUrl: "",
  hiddenInvalidNetwork: false,
};

export function useHeadStateNew() {
  const { profiles } = useProfiles();
  const current = profiles?.current || "";

  const [state, setState] = useState<Record<string, HeadState>>({});

  useEffect(() => {
    if (!current) {
      setState({});
      return;
    }

    try {
      const data = JSON.parse(
        localStorage.getItem(HEAD_STATE_KEY)!
      ) as HeadStateStorage;

      const value = data[current] || {};

      if (value && typeof value === "object") {
        setState(value);
      } else {
        setState({});
      }
    } catch {}
  }, [current]);

  const setHeadState = useCallback(
    (groupName: string, obj: Partial<HeadState>) => {
      setState((old) => {
        const state = old[groupName] || DEFAULT_STATE;
        const ret = { ...old, [groupName]: { ...state, ...obj } };

        // 保存到存储中
        setTimeout(() => {
          try {
            const item = localStorage.getItem(HEAD_STATE_KEY);

            let data = (item ? JSON.parse(item) : {}) as HeadStateStorage;

            if (!data || typeof data !== "object") data = {};

            data[current] = ret;

            localStorage.setItem(HEAD_STATE_KEY, JSON.stringify(data));
          } catch {}
        });

        return ret;
      });
    },
    [current]
  );

  return [state, setHeadState] as const;
}
