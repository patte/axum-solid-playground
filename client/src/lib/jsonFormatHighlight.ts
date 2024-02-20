// source: https://github.com/luyilin/json-format-highlight

interface Colors {
  keyColor: string;
  numberColor: string;
  stringColor: string;
  trueColor: string;
  falseColor: string;
  nullColor: string;
  curlyBraketsColor: string;
  squareBraketsColor: string;
}

const defaultColors: Colors = {
  keyColor: "#9CDCFE",
  numberColor: "#B5CEA8",
  stringColor: "#CE9178",
  trueColor: "#569CD6",
  falseColor: "#569CD6",
  nullColor: "#569CD6",
  curlyBraketsColor: "#ffce0b",
  squareBraketsColor: "#cf54cd",
};

const entityMap: { [key: string]: string } = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#39;",
  "`": "&#x60;",
  "=": "&#x3D;",
};

function escapeHtml(html: string): string {
  return String(html).replace(/[&<>"'`=]/g, function (s) {
    return entityMap[s];
  });
}

export default function (
  json: any,
  colorOptions: Partial<Colors> = {}
): string {
  let valueType: string = typeof json;
  if (valueType !== "string") {
    json = JSON.stringify(json, null, 2) || valueType;
  }
  let colors: Colors = { ...defaultColors, ...colorOptions };
  json = json.replace(/&/g, "&").replace(/</g, "<").replace(/>/g, ">");
  return json.replace(
    /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+]?\d+)?|{|}|\[|\])/g,
    (match: string): string => {
      let color: string = colors.numberColor;
      let style: string = "";
      if (/^"/.test(match)) {
        if (/:$/.test(match)) {
          color = colors.keyColor;
        } else {
          color = colors.stringColor;
          match = '"' + escapeHtml(match.substr(1, match.length - 2)) + '"';
          style = "word-wrap:break-word;white-space:pre-wrap;";
        }
      } else {
        color = /true/.test(match)
          ? colors.trueColor
          : /false/.test(match)
          ? colors.falseColor
          : /null/.test(match)
          ? colors.nullColor
          : /\{|\}/.test(match)
          ? colors.curlyBraketsColor
          : /\[|\]/.test(match)
          ? colors.squareBraketsColor
          : color;
      }
      return `<span style="${style}color:${color}">${match}</span>`;
    }
  );
}
