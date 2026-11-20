export type ShareMode = "unlimited" | "limited";

export type EditableWindow = {
  kind: "day" | "week" | "month";
  enabled: boolean;
  limit: string;
  accumulate: boolean;
  reset_time: string;
  reset_weekday: number;
  reset_day: number;
};

export type EditableShare = {
  identity_id: string;
  display_name: string;
};
