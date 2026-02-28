import { useState } from "react";
import ReactDOM from "react-dom/client";
import LandingPage from "./LandingPage";
import App from "./App";
import "./styles.css"

function Root() {
  // Si ya hay un token guardado, saltamos directo al dashboard
  const [showApp, setShowApp] = useState(() => !!localStorage.getItem("pp_token"));

  return showApp
    ? <App />
    : <LandingPage onEnterApp={() => setShowApp(true)} />;
}

ReactDOM.createRoot(document.getElementById("root")!).render(<Root />);