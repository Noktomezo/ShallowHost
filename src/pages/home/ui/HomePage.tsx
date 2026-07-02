import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import {
  AppWindow,
  Ban,
  Circle,
  GripVertical,
  Plus,
  SlidersHorizontal,
  Trash2,
} from "lucide-react";
import {
  DndContext,
  type DragEndEvent,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { restrictToVerticalAxis } from "@dnd-kit/modifiers";
import { Badge } from "@/shared/ui/badge";
import { Button } from "@/shared/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/shared/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "@/shared/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/shared/ui/select";
import { Separator } from "@/shared/ui/separator";
import { Slider } from "@/shared/ui/slider";
import { Switch } from "@/shared/ui/switch";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/shared/ui/tooltip";
import { useAudioConfigStore, type AudioConfig } from "@/shared/model/audio-config-store";
import { useChainStore, type ChainItem } from "@/shared/model/chain-store";

interface ParamInfo {
  index: number;
  name: string;
  unit: string;
  min: number;
  max: number;
  default: number;
  step_count: number;
  value: number;
}

interface AudioDevices {
  inputs: DeviceInfo[];
  outputs: DeviceInfo[];
}

interface DeviceInfo {
  name: string;
  default: boolean;
}

const SAMPLE_RATES = [44100, 48000, 88200, 96000, 192000];
const BUFFER_SIZES = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];

export function HomePage() {
  const { t } = useTranslation();
  const [error, setError] = useState<string | null>(null);
  const chain = useChainStore((s) => s.chain);
  const refreshChain = useChainStore((s) => s.refresh);
  const config = useAudioConfigStore((s) => s.config);
  const updateConfigStore = useAudioConfigStore((s) => s.updateConfig);
  const loadFromBackend = useAudioConfigStore((s) => s.loadFromBackend);
  const [devices, setDevices] = useState<AudioDevices>({ inputs: [], outputs: [] });

  useEffect(() => {
    refreshChain();
    loadFromBackend();
    invoke<AudioDevices>("get_audio_devices").then(setDevices).catch(() => {});
  }, [refreshChain, loadFromBackend]);

  // Auto-restart on config change
  async function updateConfig(patch: Partial<AudioConfig>) {
    updateConfigStore(patch);
    const next = { ...config, ...patch };
    try {
      await invoke("set_audio_config", { config: next });
      await invoke("stop_audio").catch(() => {});
      await invoke("start_audio");
    } catch (e) {
      setError(String(e));
    }
  }

  async function reorderPlugin(id: string, toIndex: number) {
    try {
      await invoke("reorder_chain", { pluginId: id, toIndex });
      await refreshChain();
    } catch (e) {
      setError(String(e));
    }
  }

  const driverItems = { wasapi: "WASAPI", asio: "ASIO" };
  const outputItems = Object.fromEntries([
    ["__default", t("home.defaultDevice")],
    ...devices.outputs.map((d) => [d.name, d.name]),
  ]);
  const inputItems = Object.fromEntries([
    ["__default", t("home.defaultDevice")],
    ...devices.inputs.map((d) => [d.name, d.name]),
  ]);
  const rateItems = Object.fromEntries(
    SAMPLE_RATES.map((r) => [
      String(r),
      r >= 1000 ? `${r / 1000} kHz` : `${r} Hz`,
    ]),
  );
  const bufferItems = Object.fromEntries(
    BUFFER_SIZES.map((b) => [
      String(b),
      <>
        {b}
        <span className="ml-1 text-muted-foreground">
          ({(b / config.sample_rate * 1000).toFixed(1)} ms)
        </span>
      </>,
    ]),
  );

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
  );

  function handleDragEnd(e: DragEndEvent) {
    const { active, over } = e;
    if (!over || active.id === over.id) return;
    const toIndex = chain.findIndex((p) => p.id === over.id);
    if (toIndex < 0) return;
    reorderPlugin(active.id as string, toIndex);
  }

  return (
    <div className="flex flex-col gap-0.5">
      <h1 className="text-xl font-semibold">{t("home.title")}</h1>
      <p className="text-sm text-muted-foreground">{t("home.description")}</p>

      <Card className="mt-3 w-full">
        <CardHeader className="gap-0.5">
          <CardTitle>{t("home.audio")}</CardTitle>
          <CardDescription>{t("home.audioDescription")}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col gap-2">
            <div className="flex items-center justify-between gap-2">
              <div className="flex flex-col gap-0">
                <span className="text-sm font-medium">{t("home.driver")}</span>
                <span className="text-xs text-muted-foreground">
                  {t("home.driverDescription")}
                </span>
              </div>
              <Select
                value={config.driver}
                onValueChange={(v) => updateConfig({ driver: v as string })}
                items={driverItems}
              >
                <SelectTrigger className="w-40">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="wasapi">WASAPI</SelectItem>
                  <SelectItem value="asio" disabled>
                    ASIO
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>

            <Separator />

            <div className="flex items-center justify-between gap-2">
              <div className="flex flex-col gap-0">
                <span className="text-sm font-medium">{t("home.outputDevice")}</span>
                <span className="text-xs text-muted-foreground">
                  {t("home.outputDeviceDescription")}
                </span>
              </div>
              <Select
                value={config.output_device ?? "__default"}
                onValueChange={(v) =>
                  updateConfig({ output_device: v === "__default" ? null : v })
                }
                items={outputItems}
              >
                <SelectTrigger className="w-40">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="__default">{t("home.defaultDevice")}</SelectItem>
                  {devices.outputs.map((d) => (
                    <SelectItem key={d.name} value={d.name}>
                      {d.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <Separator />

            <div className="flex items-center justify-between gap-2">
              <div className="flex flex-col gap-0">
                <span className="text-sm font-medium">{t("home.inputDevice")}</span>
                <span className="text-xs text-muted-foreground">
                  {t("home.inputDeviceDescription")}
                </span>
              </div>
              <Select
                value={config.input_device ?? "__default"}
                onValueChange={(v) =>
                  updateConfig({ input_device: v === "__default" ? null : v })
                }
                items={inputItems}
              >
                <SelectTrigger className="w-40">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="__default">{t("home.defaultDevice")}</SelectItem>
                  {devices.inputs.map((d) => (
                    <SelectItem key={d.name} value={d.name}>
                      {d.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <Separator />

            <div className="flex items-center justify-between gap-2">
              <div className="flex flex-col gap-0">
                <span className="text-sm font-medium">{t("home.sampleRate")}</span>
                <span className="text-xs text-muted-foreground">
                  {t("home.sampleRateDescription")}
                </span>
              </div>
              <Select
                value={String(config.sample_rate)}
                onValueChange={(v) => updateConfig({ sample_rate: Number(v) })}
                items={rateItems}
              >
                <SelectTrigger className="w-40">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {SAMPLE_RATES.map((r) => (
                    <SelectItem key={r} value={String(r)}>
                      {r >= 1000 ? `${r / 1000} kHz` : `${r} Hz`}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <Separator />

            <div className="flex items-center justify-between gap-2">
              <div className="flex flex-col gap-0">
                <span className="text-sm font-medium">{t("home.bufferSize")}</span>
                <span className="text-xs text-muted-foreground">
                  {t("home.bufferSizeDescription")}
                </span>
              </div>
              <Select
                value={String(config.buffer_size)}
                onValueChange={(v) => updateConfig({ buffer_size: Number(v) })}
                items={bufferItems}
              >
                <SelectTrigger className="w-40">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {BUFFER_SIZES.map((b) => (
                    <SelectItem key={b} value={String(b)}>
                      {b}
                      <span className="ml-1 text-muted-foreground">
                        ({(b / config.sample_rate * 1000).toFixed(1)} ms)
                      </span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <Separator />

            <div className="flex items-center justify-between gap-2">
              <div className="flex flex-col gap-0">
                <span className="text-sm font-medium">{t("home.mono")}</span>
                <span className="text-xs text-muted-foreground">
                  {t("home.monoDescription")}
                </span>
              </div>
              <Switch
                checked={config.mono}
                onCheckedChange={(v) => updateConfig({ mono: v })}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {error && (
        <p className="mt-2 text-sm text-destructive">{error}</p>
      )}

      <Card className="mt-3 w-full">
        <CardHeader className="gap-0.5">
          <CardTitle>{t("home.chain")}</CardTitle>
          <CardDescription>{t("home.addHint")}</CardDescription>
        </CardHeader>
        <CardContent>
          {chain.length > 0 ? (
            <DndContext
              sensors={sensors}
              modifiers={[restrictToVerticalAxis]}
              onDragEnd={handleDragEnd}
            >
              <SortableContext items={chain.map((p) => p.id)} strategy={verticalListSortingStrategy}>
                <div className="flex flex-col gap-2">
                  {chain.map((p) => (
                    <SortableChainCard key={p.id} plugin={p} />
                  ))}
                </div>
              </SortableContext>
            </DndContext>
          ) : (
            <div className="flex items-center gap-2 pt-2 text-sm text-muted-foreground">
              <Plus className="size-4" />
              {t("home.chainEmpty")}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function SortableChainCard({ plugin: p }: { plugin: ChainItem }) {
  const { t } = useTranslation();
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: p.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  async function openGui() {
    try {
      await invoke("open_plugin_gui", { pluginId: p.id });
    } catch (e) {
      console.error(e);
    }
  }

  return (
    <Card ref={setNodeRef} size="sm" style={style}>
      <CardHeader className="gap-0.5">
        <div className="flex items-center gap-3">
          <Button
            variant="outline"
            size="icon"
            className="cursor-grab active:cursor-grabbing touch-none"
            aria-label={t("home.dragHandle")}
            {...attributes}
            {...listeners}
          >
            <GripVertical className="size-4 text-muted-foreground" />
          </Button>
          <div className="flex flex-col gap-0">
            <div className="flex items-center gap-2">
              <CardTitle className={p.bypassed ? "text-muted-foreground" : ""}>
                {p.name}
              </CardTitle>
              <Badge variant="secondary" className="shrink-0">
                {p.format.toUpperCase()}
              </Badge>
            </div>
            <CardDescription>
              {p.bypassed ? t("home.bypassed") : (p.vendor || "")}
            </CardDescription>
          </div>
        </div>
        <CardAction className="self-center">
          <div className="flex gap-1">
            <Tooltip>
              <TooltipTrigger
                render={
                  <Button
                    variant="outline"
                    size="icon"
                    aria-label={t("home.openGui")}
                    onClick={openGui}
                  >
                    <AppWindow className="size-4" />
                  </Button>
                }
              />
              <TooltipContent>{t("home.openGui")}</TooltipContent>
            </Tooltip>
            <ChainParamsButton pluginId={p.id} name={p.name} />
            <Tooltip>
              <TooltipTrigger
                render={
                  <Button
                    variant="outline"
                    size="icon"
                    className={
                      p.bypassed
                        ? "border-amber-600/30 bg-amber-600/15 text-amber-600 hover:bg-amber-600/25 dark:text-amber-500"
                        : ""
                    }
                    onClick={() => bypassPluginGlobal(p.id, p.bypassed)}
                  >
                    {p.bypassed ? (
                      <Circle className="size-4" />
                    ) : (
                      <Ban className="size-4" />
                    )}
                  </Button>
                }
              />
              <TooltipContent>{t("home.bypass")}</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger
                render={
                  <Button
                    variant="outline"
                    size="icon"
                    className="hover:bg-destructive/15 hover:text-destructive hover:border-destructive/30"
                    onClick={() => removeFromChainGlobal(p.id)}
                  >
                    <Trash2 className="size-4" />
                  </Button>
                }
              />
              <TooltipContent>{t("home.removeFromChain")}</TooltipContent>
            </Tooltip>
          </div>
        </CardAction>
      </CardHeader>
    </Card>
  );
}

function bypassPluginGlobal(id: string, bypassed: boolean) {
  invoke("bypass_plugin", { pluginId: id, bypassed: !bypassed })
    .then(() => {
      const cur = useChainStore.getState().chain;
      useChainStore.setState({
        chain: cur.map((p) => (p.id === id ? { ...p, bypassed: !bypassed } : p)),
      });
    })
    .catch(console.error);
}

function removeFromChainGlobal(id: string) {
  invoke("remove_from_chain", { pluginId: id })
    .then(() => useChainStore.getState().remove(id))
    .catch(console.error);
}

function ChainParamsButton({ pluginId, name }: { pluginId: string; name: string }) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [params, setParams] = useState<ParamInfo[]>([]);

  async function loadParams() {
    setParams([]);
    try {
      const result = await invoke<ParamInfo[]>("get_plugin_parameters", {
        pluginId,
      });
      setParams(result);
    } catch (e) {
      console.error(e);
    }
  }

  async function handleParamChange(index: number, value: number) {
    setParams((prev) =>
      prev.map((p) => (p.index === index ? { ...p, value } : p)),
    );
    try {
      await invoke("set_plugin_parameter", {
        pluginId,
        paramIndex: index,
        value,
      });
    } catch (e) {
      console.error(e);
    }
  }

  return (
    <>
      <Tooltip>
        <TooltipTrigger
          render={
            <Button
              variant="outline"
              size="icon"
              aria-label={t("home.parameters")}
              onClick={() => {
                setOpen(true);
                loadParams();
              }}
            >
              <SlidersHorizontal className="size-4" />
            </Button>
          }
        />
        <TooltipContent>{t("home.parameters")}</TooltipContent>
      </Tooltip>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="sm:max-w-lg">
          <DialogTitle>{name}</DialogTitle>
          <DialogDescription>{t("home.parameters")}</DialogDescription>
          <div className="flex max-h-[60vh] flex-col gap-3 overflow-y-auto">
            {params.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                {t("home.noParameters")}
              </p>
            ) : (
              params.map((p) => {
                const step =
                  p.step_count > 0
                    ? (p.max - p.min) / p.step_count
                    : (p.max - p.min) / 1000;
                return (
                  <div key={p.index} className="flex flex-col gap-1">
                    <div className="flex justify-between text-sm">
                      <span>{p.name}</span>
                      <span className="text-muted-foreground tabular-nums">
                        {p.value.toFixed(2)}
                        {p.unit && ` ${p.unit}`}
                      </span>
                    </div>
                    <Slider
                      value={[p.value]}
                      min={p.min}
                      max={p.max}
                      step={step}
                      onValueChange={(v) => {
                        const val = Array.isArray(v) ? v[0] : v;
                        handleParamChange(p.index, val);
                      }}
                    />
                  </div>
                );
              })
            )}
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}
