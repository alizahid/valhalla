import { useState } from "react";
import { TextInput, type KeyEvent } from "@valhalla/runtime";

export default function App() {
    const [n, setN] = useState(0);
    const [name, setName] = useState("");

    const onRootKey = (e: KeyEvent) => {
        if ((e.metaKey || e.ctrlKey) && e.key === "k") {
            console.log("hotkey: cmd/ctrl+k");
        }
    };

    return (
        <div
            className="flex flex-col items-center justify-center gap-4 p-8 size-full bg-slate-900"
            onKeyDown={onRootKey}
        >
            <div className="text-3xl text-white">Count: {n}</div>
            <div className="flex flex-row gap-2">
                <button
                    className="px-4 py-2 rounded bg-blue-500 text-white"
                    onClick={() => setN(n - 1)}
                >
                    {"−"}
                </button>
                <button
                    className="px-4 py-2 rounded bg-blue-500 text-white"
                    onClick={() => setN(n + 1)}
                >
                    +
                </button>
            </div>
            <TextInput
                className="px-2 py-1 rounded bg-white text-black"
                placeholder="Your name"
                value={name}
                onChange={(e) => setName(e.target.value)}
            />
            <div style={{ color: "#a855f7", padding: 17 }}>
                Hello, {name || "stranger"}!
            </div>
        </div>
    );
}
