interface ShellHeaderProps {
  readonly title: string;
  readonly subtitle: string;
}

export function ShellHeader({ title, subtitle }: ShellHeaderProps) {
  return (
    <header className="shell-header">
      <div>
        <p className="eyebrow">localhost control plane</p>
        <h1>{title}</h1>
        <p className="subtitle">{subtitle}</p>
      </div>
    </header>
  );
}
