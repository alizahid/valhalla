// A small Kanban-style task board. Built to push as much of the framework
// as possible: every primitive in @valhalla/runtime gets used, the layout
// is multi-column with a sidebar and detail panel, and most interactions
// are event-driven (move/delete cards, toggle settings, mark urgent).

import { useMemo, useState } from "react";
import {
    Badge,
    Button,
    Checkbox,
    Divider,
    Pressable,
    ScrollView,
    Switch,
    Text,
    View,
} from "@valhalla/runtime";

// ─── data model ──────────────────────────────────────────────────────────

type Priority = "low" | "med" | "high";
type ColumnId = "backlog" | "doing" | "done";

type Card = {
    id: number;
    title: string;
    priority: Priority;
    tags: string[];
    column: ColumnId;
    urgent: boolean;
};

const COLUMNS: { id: ColumnId; label: string; tone: string }[] = [
    { id: "backlog", label: "Backlog", tone: "bg-slate-700" },
    { id: "doing", label: "In Progress", tone: "bg-blue-700" },
    { id: "done", label: "Done", tone: "bg-green-700" },
];

const SEED: Card[] = [
    {
        id: 1,
        title: "Wire react-refresh into HMR",
        priority: "high",
        tags: ["runtime", "dx"],
        column: "doing",
        urgent: true,
    },
    {
        id: 2,
        title: "Pin gpui git rev",
        priority: "med",
        tags: ["chore"],
        column: "doing",
        urgent: false,
    },
    {
        id: 3,
        title: "TextInput cursor + IME",
        priority: "high",
        tags: ["runtime", "input"],
        column: "backlog",
        urgent: false,
    },
    {
        id: 4,
        title: "Scrollview momentum",
        priority: "low",
        tags: ["polish"],
        column: "backlog",
        urgent: false,
    },
    {
        id: 5,
        title: "Set up Cargo workspace",
        priority: "med",
        tags: ["scaffold"],
        column: "done",
        urgent: false,
    },
    {
        id: 6,
        title: "Bridge JS ops to SceneTree",
        priority: "high",
        tags: ["runtime"],
        column: "done",
        urgent: false,
    },
    {
        id: 7,
        title: "Tailwind class subset",
        priority: "med",
        tags: ["runtime", "styling"],
        column: "done",
        urgent: false,
    },
];

const SAMPLE_TITLES = [
    "Audit unused exports",
    "Try a Wayland window",
    "Add Slider primitive",
    "Hover state on Pressable",
    "Multi-line Text",
    "Dark/light theme",
    "Animation system",
];

// ─── small helpers ───────────────────────────────────────────────────────

const priorityVariant = (p: Priority) =>
    p === "high" ? "danger" : p === "med" ? "warning" : "info";

const nextColumn = (c: ColumnId): ColumnId | null =>
    c === "backlog" ? "doing" : c === "doing" ? "done" : null;
const prevColumn = (c: ColumnId): ColumnId | null =>
    c === "done" ? "doing" : c === "doing" ? "backlog" : null;

// ─── App ─────────────────────────────────────────────────────────────────

export default function App() {
    const [cards, setCards] = useState<Card[]>(SEED);
    const [selectedId, setSelectedId] = useState<number | null>(1);
    const [showDone, setShowDone] = useState(true);
    const [sortByPriority, setSortByPriority] = useState(false);
    const [nextId, setNextId] = useState(SEED.length + 1);
    const [sampleIdx, setSampleIdx] = useState(0);

    const visible = useMemo(() => {
        let list = cards;
        if (!showDone) list = list.filter((c) => c.column !== "done");
        if (sortByPriority) {
            const order: Record<Priority, number> = { high: 0, med: 1, low: 2 };
            list = [...list].sort((a, b) => order[a.priority] - order[b.priority]);
        }
        return list;
    }, [cards, showDone, sortByPriority]);

    const selected = useMemo(
        () => cards.find((c) => c.id === selectedId) ?? null,
        [cards, selectedId],
    );

    const update = (id: number, patch: Partial<Card>) => {
        setCards((cs) => cs.map((c) => (c.id === id ? { ...c, ...patch } : c)));
    };
    const remove = (id: number) => {
        setCards((cs) => cs.filter((c) => c.id !== id));
        if (selectedId === id) setSelectedId(null);
    };
    const addCard = (column: ColumnId) => {
        const title = SAMPLE_TITLES[sampleIdx % SAMPLE_TITLES.length];
        setSampleIdx((i) => i + 1);
        const id = nextId;
        setNextId((n) => n + 1);
        setCards((cs) => [
            ...cs,
            {
                id,
                title,
                priority: "med",
                tags: ["new"],
                column,
                urgent: false,
            },
        ]);
        setSelectedId(id);
    };

    const counts = useMemo(() => {
        const c = { backlog: 0, doing: 0, done: 0 };
        for (const card of cards) c[card.column]++;
        return c;
    }, [cards]);

    return (
        <View className="flex flex-col size-full bg-slate-900">
            <Header
                total={cards.length}
                showDone={showDone}
                onShowDoneChange={setShowDone}
                sortByPriority={sortByPriority}
                onSortChange={setSortByPriority}
            />

            <Divider />

            <View className="flex flex-row gap-4 p-4 size-full">
                {/* ── columns ─────────────────────────────────────── */}
                <View className="flex flex-row gap-4 size-full">
                    {COLUMNS.map((col) => {
                        if (!showDone && col.id === "done") return null;
                        const cardsHere = visible.filter((c) => c.column === col.id);
                        return (
                            <Column
                                key={col.id}
                                id={col.id}
                                label={col.label}
                                tone={col.tone}
                                count={counts[col.id]}
                                cards={cardsHere}
                                selectedId={selectedId}
                                onSelect={setSelectedId}
                                onMove={(card, dir) => {
                                    const target =
                                        dir === "next"
                                            ? nextColumn(card.column)
                                            : prevColumn(card.column);
                                    if (target) update(card.id, { column: target });
                                }}
                                onDelete={remove}
                                onAdd={addCard}
                            />
                        );
                    })}
                </View>

                {/* ── detail panel ────────────────────────────────── */}
                <DetailPanel
                    card={selected}
                    onChangePriority={(p) =>
                        selected && update(selected.id, { priority: p })
                    }
                    onToggleUrgent={(v) =>
                        selected && update(selected.id, { urgent: v })
                    }
                    onClose={() => setSelectedId(null)}
                />
            </View>
        </View>
    );
}

// ─── Header ──────────────────────────────────────────────────────────────

function Header(props: {
    total: number;
    showDone: boolean;
    onShowDoneChange: (v: boolean) => void;
    sortByPriority: boolean;
    onSortChange: (v: boolean) => void;
}) {
    return (
        <View className="flex flex-row items-center justify-between px-6 py-4 bg-slate-800">
            <View className="flex flex-row items-center gap-3">
                <Text className="text-2xl text-white">Valhalla</Text>
                <Badge variant="info">{props.total} tasks</Badge>
            </View>
            <View className="flex flex-row items-center gap-4">
                <Switch
                    checked={props.showDone}
                    onValueChange={props.onShowDoneChange}
                    label="Show done"
                />
                <Switch
                    checked={props.sortByPriority}
                    onValueChange={props.onSortChange}
                    label="Sort by priority"
                />
            </View>
        </View>
    );
}

// ─── Column ──────────────────────────────────────────────────────────────

function Column(props: {
    id: ColumnId;
    label: string;
    tone: string;
    count: number;
    cards: Card[];
    selectedId: number | null;
    onSelect: (id: number) => void;
    onMove: (card: Card, dir: "next" | "prev") => void;
    onDelete: (id: number) => void;
    onAdd: (column: ColumnId) => void;
}) {
    return (
        <View className="flex flex-col gap-2 size-full bg-slate-800 rounded-lg p-3">
            <View className="flex flex-row items-center justify-between">
                <View className="flex flex-row items-center gap-2">
                    <View className={`w-2 h-2 rounded-full ${props.tone}`} />
                    <Text className="text-white text-lg">{props.label}</Text>
                    <Badge>{props.count}</Badge>
                </View>
                <Button
                    label="+ Add"
                    variant="ghost"
                    size="sm"
                    onPress={() => props.onAdd(props.id)}
                />
            </View>

            <Divider />

            <ScrollView className="flex flex-col gap-2 size-full">
                {props.cards.length === 0 ? (
                    <View className="p-6">
                        <Text className="text-slate-500">No tasks here yet.</Text>
                    </View>
                ) : (
                    props.cards.map((card) => (
                        <CardRow
                            key={card.id}
                            card={card}
                            selected={card.id === props.selectedId}
                            onSelect={() => props.onSelect(card.id)}
                            onMovePrev={() => props.onMove(card, "prev")}
                            onMoveNext={() => props.onMove(card, "next")}
                            onDelete={() => props.onDelete(card.id)}
                        />
                    ))
                )}
            </ScrollView>
        </View>
    );
}

// ─── CardRow ─────────────────────────────────────────────────────────────

function CardRow(props: {
    card: Card;
    selected: boolean;
    onSelect: () => void;
    onMovePrev: () => void;
    onMoveNext: () => void;
    onDelete: () => void;
}) {
    const ringClass = props.selected
        ? "border-2 border-blue-400"
        : "border border-slate-700";
    const urgentClass = props.card.urgent ? "bg-red-900" : "bg-slate-900";

    return (
        <Pressable
            className={`flex flex-col gap-2 p-3 rounded-md ${ringClass} ${urgentClass}`}
            onPress={props.onSelect}
        >
            <View className="flex flex-row items-start justify-between gap-2">
                <Text className="text-white text-base">{props.card.title}</Text>
                <Badge variant={priorityVariant(props.card.priority)}>
                    {props.card.priority}
                </Badge>
            </View>
            {props.card.tags.length > 0 && (
                <View className="flex flex-row gap-1">
                    {props.card.tags.map((t) => (
                        <Badge key={t}>#{t}</Badge>
                    ))}
                </View>
            )}
            <View className="flex flex-row items-center justify-between">
                <View className="flex flex-row gap-1">
                    <Button
                        label="←"
                        variant="ghost"
                        size="xs"
                        onPress={props.onMovePrev}
                    />
                    <Button
                        label="→"
                        variant="ghost"
                        size="xs"
                        onPress={props.onMoveNext}
                    />
                </View>
                <Button
                    label="Delete"
                    variant="danger"
                    size="xs"
                    onPress={props.onDelete}
                />
            </View>
        </Pressable>
    );
}

// ─── DetailPanel ─────────────────────────────────────────────────────────

function DetailPanel(props: {
    card: Card | null;
    onChangePriority: (p: Priority) => void;
    onToggleUrgent: (v: boolean) => void;
    onClose: () => void;
}) {
    if (!props.card) {
        return (
            <View className="flex flex-col items-center justify-center w-72 bg-slate-800 rounded-lg p-6">
                <Text className="text-slate-400">Select a card to see details.</Text>
            </View>
        );
    }
    const card = props.card;

    return (
        <View className="flex flex-col gap-3 w-72 bg-slate-800 rounded-lg p-4">
            <View className="flex flex-row items-center justify-between">
                <Text className="text-white text-lg">Details</Text>
                <Button
                    label="×"
                    variant="ghost"
                    size="xs"
                    onPress={props.onClose}
                />
            </View>
            <Divider />

            <View className="flex flex-col gap-1">
                <Text className="text-slate-400 text-xs">Title</Text>
                <Text className="text-white text-base">{card.title}</Text>
            </View>

            <View className="flex flex-col gap-1">
                <Text className="text-slate-400 text-xs">Column</Text>
                <Text className="text-white">{card.column}</Text>
            </View>

            <View className="flex flex-col gap-2">
                <Text className="text-slate-400 text-xs">Priority</Text>
                <View className="flex flex-row gap-1">
                    {(["low", "med", "high"] as Priority[]).map((p) => (
                        <Button
                            key={p}
                            label={p}
                            variant={card.priority === p ? "primary" : "ghost"}
                            size="xs"
                            onPress={() => props.onChangePriority(p)}
                        />
                    ))}
                </View>
            </View>

            <Divider />

            <Checkbox
                checked={card.urgent}
                onValueChange={props.onToggleUrgent}
                label="Urgent"
            />

            <View className="flex flex-col gap-1" style={{ marginTop: 12 }}>
                <Text className="text-slate-400 text-xs">Tags</Text>
                <View className="flex flex-row gap-1">
                    {card.tags.map((t) => (
                        <Badge key={t}>#{t}</Badge>
                    ))}
                </View>
            </View>
        </View>
    );
}
