interface ShellHeaderProps {
  readonly title: string;
  readonly subtitle?: string;
}

export function ShellHeader({ title, subtitle }: ShellHeaderProps) {
  return (
    <div className="titlebar-brand">
      <p className="eyebrow">localhost control plane</p>
      <h1 className="titlebar-title">{title}</h1>
      {subtitle ? <span className="sr-only">{subtitle}</span> : null}
    </div>
  );
}
