import type {Route} from "./+types/home";
import TerminalApp from "~/components/TerminalApp";

export function meta({}: Route.MetaArgs) {
  return [
    {title: "Mini Launchpad"},
  ];
}

export default function Home() {
  return (
    <main className="page" style={{padding: "2rem"}}>
      <TerminalApp/>
    </main>
  );
}
