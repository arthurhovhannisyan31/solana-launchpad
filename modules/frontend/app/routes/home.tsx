import type {Route} from "./+types/home";
import TerminalApp from "~/components/TerminalApp";


export function meta({}: Route.MetaArgs) {
  return [
    {title: "Mini Launchpad"},
  ];
}

export default function Home() {
  // const [ClientTerminal, setClientTerminal] = useState<ComponentType | null>(null);

  // useEffect(() => {
  //   import("../components/TerminalApp").then((m) => setClientTerminal(() => m.default));
  // }, []);

  // console.log({
  //   t: ClientTerminal
  // })

  // if (!ClientTerminal) {
  //   return (
  //     <main className="page" style={{padding: "2rem"}}>
  //       <div className="terminal">
  //         <div className="terminal-header">
  //           <span className="terminal-title">mini-launchpad@localnet</span>
  //         </div>
  //         <pre className="terminal-body">
  //           <span className="term-line term-muted">загрузка...</span>
  //         </pre>
  //       </div>
  //     </main>
  //   );
  // }

  return (
    <main className="page" style={{padding: "2rem"}}>
      <TerminalApp/>
    </main>
  );
}
