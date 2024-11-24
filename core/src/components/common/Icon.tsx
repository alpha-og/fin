import { Folder, File, FileQuestion, Copy } from "lucide-react";
function Icon({ icon, size }: { icon: string | null; size: number }) {
  if (icon === null) return <FileQuestion size={size} className="shrink-0" />;
  switch (icon.toLowerCase()) {
    case "application":
      return <File size={size} className="shrink-0" />;
    case "folder":
      return <Folder size={size} className="shrink-0" />;
    case "file":
      return <File size={size} className="shrink-0" />;
    case "copy":
      return <Copy size={size} className="shrink-0" />;
    default:
      return <FileQuestion size={size} className="shrink-0" />;
  }
}

export default Icon;
