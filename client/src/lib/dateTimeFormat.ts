const defaultDateTimeFormat: Intl.DateTimeFormatOptions = {
  year: "numeric",
  month: "numeric",
  day: "numeric",
  hour: "numeric",
  minute: "numeric",
};

// get a date string
// example:
// 6/30/2023
export const createToLocaleDateString = (
  date: Date | string,
  options: Intl.DateTimeFormatOptions = defaultDateTimeFormat,
  timeZoneIANA: string | undefined
) =>
  new Date(date).toLocaleDateString(undefined, {
    ...options,
    hour: undefined,
    minute: undefined,
    ...(timeZoneIANA ? { timeZone: timeZoneIANA } : {}),
  });

// get a relative time string
// examples:
// this minute
// in 1 min.
// 1 min. ago
// in 2 hr.
// 2 hr. ago
// tomorrow
// yesterday
// in 2 days
// 2 days ago
// in 3 mo. (9/28/2023)
// 3 mo. ago (4/1/2023)
// next yr. (8/23/2024)
// last yr. (8/23/2022)
export const toLocaleRelativeTimeString = (
  now: Date,
  date: Date | string,
  showFullDateForOldDates = true
) => {
  const rtf = new Intl.RelativeTimeFormat(undefined, {
    numeric: "auto",
    style: "short",
  });
  const diff = now.getTime() - new Date(date).getTime();

  const diffMinutes = Math.trunc(diff / 1000 / 60);
  const diffHours = Math.trunc(diff / 1000 / 60 / 60);
  const diffDays = Math.trunc(diff / 1000 / 60 / 60 / 24);
  if (Math.abs(diffMinutes) < 60) {
    return rtf.format(-diffMinutes, "minute");
  }
  if (Math.abs(diffHours) < 24) {
    return rtf.format(-diffHours, "hour");
  }
  if (Math.abs(diffDays) < 30) {
    return rtf.format(-diffDays, "day");
  }

  const diffMonths = Math.trunc(diff / 1000 / 60 / 60 / 24 / 30);
  const diffYears = Math.trunc(diff / 1000 / 60 / 60 / 24 / 30 / 12);
  const dateTimeString = showFullDateForOldDates
    ? ` (${createToLocaleDateString(date, defaultDateTimeFormat, undefined)})`
    : "";
  if (Math.abs(diffMonths) < 12) {
    return `${rtf.format(-diffMonths, "month")}${dateTimeString}`;
  }
  return `${rtf.format(-diffYears, "year")}${dateTimeString}`;
};
