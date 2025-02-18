import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Box, IconButton, TextField, SxProps } from "@mui/material";
import {
  AccessTimeRounded,
  MyLocationRounded,
  NetworkCheckRounded,
  FilterAltRounded,
  FilterAltOffRounded,
  VisibilityRounded,
  VisibilityOffRounded,
  WifiTetheringRounded,
  WifiTetheringOffRounded,
  SortByAlphaRounded,
  SortRounded,
  SignalWifiStatusbar4Bar,
  SignalWifiStatusbarConnectedNoInternet4,
} from "@mui/icons-material";
import { useVerge } from "@/hooks/use-verge";
import type { HeadState } from "./use-head-state";
import type { ProxySortType } from "./use-filter-sort";
import delayManager from "@/services/delay";

interface Props {
  sx?: SxProps;
  url?: string;
  groupName: string;
  headState: HeadState;
  onLocation: () => void;
  onCheckDelay: () => void;
  onHeadState: (val: Partial<HeadState>) => void;
}

export const ProxyHead = (props: Props) => {
  const { sx = {}, url, groupName, headState, onHeadState } = props;

  const {
    hiddenInvalidNetwork,
    showType,
    sortType,
    filterText,
    textState,
    testUrl,
  } = headState;

  const { t } = useTranslation();
  const [autoFocus, setAutoFocus] = useState(false);

  useEffect(() => {
    // fix the focus conflict
    const timer = setTimeout(() => setAutoFocus(true), 100);
    return () => clearTimeout(timer);
  }, []);

  const { verge } = useVerge();

  useEffect(() => {
    delayManager.setUrl(
      groupName,
      testUrl || url || verge?.default_latency_test!
    );
  }, [groupName, testUrl, verge?.default_latency_test]);

  return (
    <Box sx={{ display: "flex", alignItems: "center", gap: 0.5, ...sx }}>
      <IconButton
        size="small"
        color="inherit"
        title={
          [
            t("Enable Hidden Invalid Network"),
            t("Disable Hidden Invalid Network"),
          ][hiddenInvalidNetwork ? 0 : 1]
        }
        onClick={() =>
          onHeadState({ hiddenInvalidNetwork: !hiddenInvalidNetwork })
        }
      >
        {hiddenInvalidNetwork && (
          <SignalWifiStatusbar4Bar style={{ fill: "#0072ea" }} />
        )}
        {!hiddenInvalidNetwork && <SignalWifiStatusbarConnectedNoInternet4 />}
      </IconButton>

      <IconButton
        size="small"
        color="inherit"
        title={t("Location")}
        onClick={props.onLocation}
      >
        <MyLocationRounded />
      </IconButton>

      <IconButton
        size="small"
        color="inherit"
        title={t("Delay check")}
        onClick={() => {
          // Remind the user that it is custom test url
          if (testUrl?.trim() && textState !== "filter") {
            onHeadState({ textState: "url" });
          }
          props.onCheckDelay();
        }}
      >
        <NetworkCheckRounded />
      </IconButton>

      <IconButton
        size="small"
        color="inherit"
        title={
          [t("Sort by default"), t("Sort by delay"), t("Sort by name")][
            sortType
          ]
        }
        onClick={() =>
          onHeadState({ sortType: ((sortType + 1) % 3) as ProxySortType })
        }
      >
        {sortType !== 1 && sortType !== 2 && <SortRounded />}
        {sortType === 1 && <AccessTimeRounded />}
        {sortType === 2 && <SortByAlphaRounded />}
      </IconButton>

      <IconButton
        size="small"
        color="inherit"
        title={t("Delay check URL")}
        onClick={() =>
          onHeadState({ textState: textState === "url" ? null : "url" })
        }
      >
        {textState === "url" ? (
          <WifiTetheringRounded />
        ) : (
          <WifiTetheringOffRounded />
        )}
      </IconButton>

      <IconButton
        size="small"
        color="inherit"
        title={showType ? t("Proxy basic") : t("Proxy detail")}
        onClick={() => onHeadState({ showType: !showType })}
      >
        {showType ? <VisibilityRounded /> : <VisibilityOffRounded />}
      </IconButton>

      <IconButton
        size="small"
        color="inherit"
        title={t("Filter")}
        onClick={() =>
          onHeadState({ textState: textState === "filter" ? null : "filter" })
        }
      >
        {textState === "filter" ? (
          <FilterAltRounded />
        ) : (
          <FilterAltOffRounded />
        )}
      </IconButton>

      {textState === "filter" && (
        <TextField
          autoComplete="new-password"
          autoFocus={autoFocus}
          hiddenLabel
          value={filterText}
          size="small"
          variant="outlined"
          placeholder={t("Filter conditions")}
          onChange={(e) => onHeadState({ filterText: e.target.value })}
          sx={{ ml: 0.5, flex: "1 1 auto", input: { py: 0.65, px: 1 } }}
        />
      )}

      {textState === "url" && (
        <TextField
          autoComplete="new-password"
          autoFocus={autoFocus}
          hiddenLabel
          autoSave="off"
          value={testUrl}
          size="small"
          variant="outlined"
          placeholder={t("Delay check URL")}
          onChange={(e) => onHeadState({ testUrl: e.target.value })}
          sx={{ ml: 0.5, flex: "1 1 auto", input: { py: 0.65, px: 1 } }}
        />
      )}
    </Box>
  );
};
