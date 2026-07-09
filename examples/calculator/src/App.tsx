// Four-function calculator built from Valhalla primitives.
//
// Everything is View / Text / Pressable + Tailwind classes. The arithmetic
// is a classic calculator state machine: an accumulator, a pending
// operator, and an "overwrite" flag that decides whether the next digit
// starts a fresh number or appends.

import { useState } from "react";
import { Pressable, Text, View } from "@valhalla/runtime";

type Op = "÷" | "×" | "−" | "+";

const MAX_DIGITS = 12;

function compute(a: number, op: Op, b: number): number {
    switch (op) {
        case "÷":
            return a / b;
        case "×":
            return a * b;
        case "−":
            return a - b;
        case "+":
            return a + b;
    }
}

// Trim float noise (0.1 + 0.2) and clamp width so the display never overflows.
function format(n: number): string {
    if (!Number.isFinite(n)) return "Error";
    const s = parseFloat(n.toPrecision(12)).toString();
    return s.length > MAX_DIGITS + 2 ? n.toExponential(6) : s;
}

export default function App() {
    const [display, setDisplay] = useState("0");
    const [acc, setAcc] = useState<number | null>(null);
    const [op, setOp] = useState<Op | null>(null);
    // True right after an operator / equals: the next digit replaces the
    // display instead of appending to it.
    const [overwrite, setOverwrite] = useState(true);

    const pressDigit = (d: string) => {
        if (overwrite) {
            setDisplay(d === "." ? "0." : d);
            setOverwrite(false);
            return;
        }
        if (d === "." && display.includes(".")) return;
        if (display.replace(/[-.]/g, "").length >= MAX_DIGITS) return;
        setDisplay(display === "0" && d !== "." ? d : display + d);
    };

    const pressOp = (next: Op) => {
        const current = parseFloat(display);
        // Chained ops (2 + 3 + …) evaluate the pending pair first.
        if (op !== null && acc !== null && !overwrite) {
            const result = compute(acc, op, current);
            setAcc(result);
            setDisplay(format(result));
        } else {
            setAcc(current);
        }
        setOp(next);
        setOverwrite(true);
    };

    const pressEquals = () => {
        if (op === null || acc === null) return;
        setDisplay(format(compute(acc, op, parseFloat(display))));
        setAcc(null);
        setOp(null);
        setOverwrite(true);
    };

    const pressClear = () => {
        setDisplay("0");
        setAcc(null);
        setOp(null);
        setOverwrite(true);
    };

    const pressNegate = () => setDisplay(format(-parseFloat(display)));
    const pressPercent = () => setDisplay(format(parseFloat(display) / 100));

    const hasInput = display !== "0" || !overwrite;

    return (
        <View className="size-full flex flex-col justify-end gap-3 bg-black p-4">
            <View className="flex flex-row justify-end px-2">
                <Text className={display.length > 8 ? "text-3xl text-white" : "text-5xl text-white"}>
                    {display}
                </Text>
            </View>

            <KeyRow>
                <FnKey label={hasInput ? "C" : "AC"} onPress={pressClear} />
                <FnKey label="±" onPress={pressNegate} />
                <FnKey label="%" onPress={pressPercent} />
                <OpKey label="÷" active={op === "÷" && overwrite} onPress={() => pressOp("÷")} />
            </KeyRow>
            <KeyRow>
                <DigitKey label="7" onPress={pressDigit} />
                <DigitKey label="8" onPress={pressDigit} />
                <DigitKey label="9" onPress={pressDigit} />
                <OpKey label="×" active={op === "×" && overwrite} onPress={() => pressOp("×")} />
            </KeyRow>
            <KeyRow>
                <DigitKey label="4" onPress={pressDigit} />
                <DigitKey label="5" onPress={pressDigit} />
                <DigitKey label="6" onPress={pressDigit} />
                <OpKey label="−" active={op === "−" && overwrite} onPress={() => pressOp("−")} />
            </KeyRow>
            <KeyRow>
                <DigitKey label="1" onPress={pressDigit} />
                <DigitKey label="2" onPress={pressDigit} />
                <DigitKey label="3" onPress={pressDigit} />
                <OpKey label="+" active={op === "+" && overwrite} onPress={() => pressOp("+")} />
            </KeyRow>
            <KeyRow>
                <DigitKey label="0" wide onPress={pressDigit} />
                <DigitKey label="." onPress={pressDigit} />
                <OpKey label="=" onPress={pressEquals} />
            </KeyRow>
        </View>
    );
}

// ─── userland: keypad widgets ─────────────────────────────────────────────

function KeyRow({ children }: { children: React.ReactNode }) {
    return <View className="flex flex-row gap-3">{children}</View>;
}

function Key(props: {
    label: string;
    className: string;
    wide?: boolean;
    onPress: () => void;
}) {
    return (
        <Pressable
            className={`${props.wide ? "w-35" : "w-16"} h-16 rounded-full flex items-center justify-center ${props.className}`}
            onPress={props.onPress}
        >
            <Text className="text-2xl">{props.label}</Text>
        </Pressable>
    );
}

function DigitKey(props: { label: string; wide?: boolean; onPress: (d: string) => void }) {
    return (
        <Key
            label={props.label}
            wide={props.wide}
            className="bg-gray-800 text-white"
            onPress={() => props.onPress(props.label)}
        />
    );
}

function FnKey(props: { label: string; onPress: () => void }) {
    return <Key label={props.label} className="bg-gray-400 text-black" onPress={props.onPress} />;
}

function OpKey(props: { label: string; active?: boolean; onPress: () => void }) {
    return (
        <Key
            label={props.label}
            className={props.active ? "bg-white text-orange-500" : "bg-orange-500 text-white"}
            onPress={props.onPress}
        />
    );
}
