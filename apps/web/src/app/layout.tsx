import "./globals.css";

export const metadata = {
  title: "AI Analytics Dashboard",
  description: "Single-tenant open-source analytics system for AI usage tracking",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
