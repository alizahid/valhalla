// Todo list built from Valhalla primitives.
//
// View / Text / Pressable / ScrollView + Tailwind. The checkbox, filter
// tabs, and delete button are all userland widgets composed from
// primitives — the framework ships no opinionated components.
//
// Free-typing into a TextInput is the framework's known V1 gap (GPUI
// focus/IME wiring is on the roadmap), so new tasks come from a
// suggestion queue instead. Everything else — toggling, deleting,
// filtering, clearing — is ordinary React state.

import { useMemo, useState } from "react";
import { Pressable, ScrollView, Text, View } from "@valhalla/runtime";

type Todo = { id: number; label: string; done: boolean };
type Filter = "all" | "active" | "done";

const SUGGESTIONS = [
    "Water the plants",
    "Reply to Freya's email",
    "Book flights to Oslo",
    "Fix the squeaky door",
    "Read the GPUI docs",
    "Sharpen the axe",
    "Feed the ravens",
    "Polish the shield",
    "Chart a course to Vinland",
    "Sweep the mead hall",
];

const INITIAL: Todo[] = [
    { id: 1, label: "Build a React renderer for GPUI", done: true },
    { id: 2, label: "Ship the calculator demo", done: true },
    { id: 3, label: "Ship the todo demo", done: false },
    { id: 4, label: "Wire up real text input", done: false },
];

export default function App() {
    const [todos, setTodos] = useState<Todo[]>(INITIAL);
    const [filter, setFilter] = useState<Filter>("all");
    const [nextSuggestion, setNextSuggestion] = useState(0);
    const [nextId, setNextId] = useState(INITIAL.length + 1);

    const addTodo = (label: string) => {
        setTodos([...todos, { id: nextId, label, done: false }]);
        setNextId(nextId + 1);
    };

    const addSuggestion = () => {
        addTodo(SUGGESTIONS[nextSuggestion % SUGGESTIONS.length]);
        setNextSuggestion(nextSuggestion + 1);
    };

    const toggle = (id: number) =>
        setTodos(todos.map((t) => (t.id === id ? { ...t, done: !t.done } : t)));

    const remove = (id: number) => setTodos(todos.filter((t) => t.id !== id));

    const clearDone = () => setTodos(todos.filter((t) => !t.done));

    const visible = useMemo(
        () =>
            filter === "all"
                ? todos
                : todos.filter((t) => (filter === "done" ? t.done : !t.done)),
        [todos, filter],
    );

    const remaining = todos.filter((t) => !t.done).length;
    const doneCount = todos.length - remaining;

    return (
        <View className="size-full flex flex-col bg-slate-900 p-6 gap-4">
            {/* header */}
            <View className="flex flex-row items-center justify-between">
                <Text className="text-3xl text-white font-bold">Todos</Text>
                <Text className="text-sm text-slate-400">
                    {remaining === 0 ? "All done ✨" : `${remaining} left`}
                </Text>
            </View>

            {/* "input" row — a suggestion feeder until TextInput can type */}
            <Pressable
                className="flex flex-row items-center justify-between rounded-lg bg-slate-800 border border-slate-700 px-4 py-3"
                onPress={addSuggestion}
            >
                <Text className="text-slate-500">
                    {SUGGESTIONS[nextSuggestion % SUGGESTIONS.length]}
                </Text>
                <View className="rounded-md bg-blue-600 px-3 py-1">
                    <Text className="text-white text-sm">Add</Text>
                </View>
            </Pressable>

            {/* filter tabs */}
            <View className="flex flex-row gap-2">
                <FilterTab label="All" value="all" current={filter} onSelect={setFilter} />
                <FilterTab label="Active" value="active" current={filter} onSelect={setFilter} />
                <FilterTab label="Done" value="done" current={filter} onSelect={setFilter} />
            </View>

            {/* list */}
            <ScrollView className="flex flex-col gap-2 h-full">
                {visible.length === 0 ? (
                    <View className="flex items-center justify-center p-8">
                        <Text className="text-slate-600">Nothing here.</Text>
                    </View>
                ) : (
                    visible.map((todo) => (
                        <TodoRow
                            key={todo.id}
                            todo={todo}
                            onToggle={() => toggle(todo.id)}
                            onDelete={() => remove(todo.id)}
                        />
                    ))
                )}
            </ScrollView>

            {/* footer */}
            <View className="flex flex-row items-center justify-between">
                <Text className="text-sm text-slate-500">
                    {`${todos.length} total · ${doneCount} done`}
                </Text>
                {doneCount > 0 && (
                    <Pressable className="rounded-md bg-slate-800 px-3 py-1" onPress={clearDone}>
                        <Text className="text-sm text-slate-300">Clear completed</Text>
                    </Pressable>
                )}
            </View>
        </View>
    );
}

// ─── userland: widgets ────────────────────────────────────────────────────

function FilterTab(props: {
    label: string;
    value: Filter;
    current: Filter;
    onSelect: (f: Filter) => void;
}) {
    const active = props.current === props.value;
    return (
        <Pressable
            className={`rounded-full px-3 py-1 ${active ? "bg-blue-600" : "bg-slate-800"}`}
            onPress={() => props.onSelect(props.value)}
        >
            <Text className={`text-sm ${active ? "text-white" : "text-slate-400"}`}>
                {props.label}
            </Text>
        </Pressable>
    );
}

function TodoRow(props: { todo: Todo; onToggle: () => void; onDelete: () => void }) {
    const { todo } = props;
    return (
        <View className="flex flex-row items-center gap-3 rounded-lg bg-slate-800 px-4 py-3">
            {/* checkbox */}
            <Pressable
                className={`w-5 h-5 rounded flex items-center justify-center ${
                    todo.done ? "bg-green-600" : "border border-slate-500"
                }`}
                onPress={props.onToggle}
            >
                {todo.done && <Text className="text-white text-xs">✓</Text>}
            </Pressable>

            {/* label — flex-1 pushes the delete button to the edge */}
            <Pressable className="flex-1" onPress={props.onToggle}>
                <Text className={todo.done ? "text-slate-500" : "text-slate-100"}>
                    {todo.label}
                </Text>
            </Pressable>

            {/* delete */}
            <Pressable
                className="w-6 h-6 rounded flex items-center justify-center bg-slate-700"
                onPress={props.onDelete}
            >
                <Text className="text-slate-400 text-xs">✕</Text>
            </Pressable>
        </View>
    );
}
