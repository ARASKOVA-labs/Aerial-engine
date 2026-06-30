import React, { useEffect, useRef, useState } from 'react';
import {
  MousePointer2,
  Pen,
  Square,
  Circle,
  Minus,
  Hand,
  ArrowUpRight,
  Highlighter,
  Moon,
  Sun,
  ZoomIn,
  ZoomOut,
  Wand2,
  Trash2,
  Eraser
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/tauri';
import './App.css';

const STROKE_COLORS = ['#1a1a2e', '#6366f1', '#f43f5e', '#10b981', '#f59e0b', '#0ea5e9'];

type ToolId = 'select' | 'freedraw' | 'fountain' | 'rectangle' | 'ellipse' | 'line' | 'arrow' | 'hand' | 'highlighter' | 'laser_pen' | 'eraser';

// Global flag to prevent double-initialization of Wasm in React StrictMode
let wasmInitPromise: Promise<any> | null = null;

export default function App() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const engineRef = useRef<any>(null); // any used since we removed full types to keep it simple

  const [activeTool, setActiveTool] = useState<ToolId>('freedraw');
  const [darkMode, setDarkMode] = useState(() => localStorage.getItem('aerial_dark_mode') === 'true');
  const [strokeColor, setStrokeColor] = useState('#1a1a2e');
  const [strokeWidth, setStrokeWidth] = useState(2.5);
  const [showColorPicker, setShowColorPicker] = useState(false);
  const [engineReady, setEngineReady] = useState(false);
  
  // UI overlays
  const [eraserPos, setEraserPos] = useState<{x: number; y: number} | null>(null);
  const activeToolRef = useRef<ToolId>('freedraw');

  useEffect(() => {
    async function loadEngine() {
      try {
        const timestamp = Date.now();
        const glueUrl  = new URL(`/aerial-engine/aerial_engine.js?v=${timestamp}`, import.meta.url).href;
        const wasmUrl  = new URL(`/aerial-engine/aerial_engine_bg.wasm?v=${timestamp}`, import.meta.url).href;

        // @ts-ignore
        const mod = await import(/* @vite-ignore */ glueUrl);
        
        if (!wasmInitPromise) {
          wasmInitPromise = mod.default({ module_or_path: wasmUrl });
        }
        await wasmInitPromise;
        await document.fonts.ready;

        if (canvasRef.current && containerRef.current) {
          const canvas = canvasRef.current;
          canvas.width = containerRef.current.clientWidth;
          canvas.height = containerRef.current.clientHeight;

          const engine = new mod.AerialCanvas('aerial-canvas');
          engineRef.current = engine;
          
          engine.set_dark_mode(darkMode);
          engine.set_stroke_color('#1a1a2e');
          engine.set_stroke_width(strokeWidth);
          engine.set_grid_type('dots');
          engine.set_tool_freedraw();
          
          // Load CRDT state from Tauri Database
          try {
            const dbBytes = await invoke<number[] | null>('load_board');
            if (dbBytes) {
              engine.import_full_state(new Uint8Array(dbBytes));
              console.log("Loaded board state from local database.");
            }
          } catch(e) {
            console.error("Failed to load local board:", e);
          }

          engine.render();
          setEngineReady(true);
        }
      } catch (err) {
        console.error('Failed to initialize aerial-engine WASM:', err);
      }
    }
    loadEngine();
    return () => {
      engineRef.current?.free?.();
    };
  }, []);

  // ── Animation Loop ─────────────────────────────────────────────────────────
  useEffect(() => {
    if (!engineReady) return;
    let animationFrameId: number;
    const loop = () => {
      const e = engineRef.current;
      if (e && e.tick_animations) {
         e.tick_animations();
      }
      animationFrameId = requestAnimationFrame(loop);
    };
    loop();
    return () => cancelAnimationFrame(animationFrameId);
  }, [engineReady]);

  // Native resize handler
  useEffect(() => {
    const handleResize = () => {
      if (canvasRef.current && engineRef.current && containerRef.current) {
        const rect = containerRef.current.getBoundingClientRect();
        canvasRef.current.width = rect.width;
        canvasRef.current.height = rect.height;
        engineRef.current.render();
      }
    };
    
    const observer = new ResizeObserver(handleResize);
    if (containerRef.current) observer.observe(containerRef.current);
    window.addEventListener('resize', handleResize);
    
    return () => {
      observer.disconnect();
      window.removeEventListener('resize', handleResize);
    };
  }, [engineReady]);

  // Auto-Save Loop
  useEffect(() => {
    if (!engineReady || !engineRef.current) return;
    
    const interval = setInterval(async () => {
      const e = engineRef.current;
      if (!e) return;
      
      const needsSave = e.check_and_clear_dirty();
      if (needsSave) {
        const stateBytes = e.export_full_state();
        try {
          await invoke('save_board', { payload: Array.from(stateBytes) });
        } catch (err) {
          console.error("Auto-save failed:", err);
        }
      }
    }, 500);
    
    return () => clearInterval(interval);
  }, [engineReady]);

  const selectTool = (tool: ToolId) => {
    if (activeTool === tool && tool !== 'select' && tool !== 'hand') {
      setShowColorPicker(!showColorPicker);
    } else {
      setActiveTool(tool);
      activeToolRef.current = tool;
      setShowColorPicker(false);
    }
    
    if (!engineRef.current) return;
    
    switch (tool) {
      case 'select': engineRef.current.set_tool_select(); break;
      case 'freedraw': engineRef.current.set_tool_freedraw(); break;
      case 'fountain': engineRef.current.set_tool_fountain_pen(); break;
      case 'rectangle': engineRef.current.set_tool_rectangle(); break;
      case 'ellipse': engineRef.current.set_tool_ellipse(); break;
      case 'line': engineRef.current.set_tool_line(); break;
      case 'arrow': engineRef.current.set_tool_arrow(); break;
      case 'hand': engineRef.current.set_tool_hand(); break;
      case 'highlighter': engineRef.current.set_tool_highlighter(); break;
      case 'laser_pen': engineRef.current.set_tool_laser_pen(); break;
      case 'eraser': engineRef.current.set_tool_eraser(); break;
    }
  };

  const selectStrokeWidth = (w: number) => {
    setStrokeWidth(w);
    if (engineRef.current) {
      engineRef.current.set_stroke_width(w);
    }
  };

  const selectColor = (color: string) => {
    setStrokeColor(color);
    if (engineRef.current) {
      engineRef.current.set_stroke_color(color);
    }
    setShowColorPicker(false);
  };

  const toggleDarkMode = () => {
    const next = !darkMode;
    setDarkMode(next);
    localStorage.setItem('aerial_dark_mode', next.toString());
    
    if (next) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    if (engineRef.current) {
      engineRef.current.set_dark_mode(next);
    }
  };

  useEffect(() => {
    const cvs = canvasRef.current;
    if (!cvs || !engineReady) return;

    // Using pointer events instead of mouse events for better touch/stylus support
    const onDown = (e: PointerEvent) => {
      // Set capture so we receive move/up events even if pointer leaves canvas bounds
      cvs.setPointerCapture(e.pointerId);
      engineRef.current?.on_mouse_down(e as any);
    };
    const onMove = (e: PointerEvent) => {
      if (activeToolRef.current === 'eraser') {
        const rect = cvs.getBoundingClientRect();
        setEraserPos({ x: e.clientX - rect.left, y: e.clientY - rect.top });
      }
      engineRef.current?.on_mouse_move(e as any);
    };
    const onUp = (e: PointerEvent) => {
      cvs.releasePointerCapture(e.pointerId);
      engineRef.current?.on_mouse_up(e as any);
    };
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      engineRef.current?.on_wheel(e.deltaX, e.deltaY, e.ctrlKey, e.clientX, e.clientY);
    };

    const onLeave = () => {
      setEraserPos(null);
    };

    cvs.addEventListener('pointerdown', onDown);
    cvs.addEventListener('pointermove', onMove);
    cvs.addEventListener('pointerup', onUp);
    cvs.addEventListener('pointercancel', onUp);
    cvs.addEventListener('pointerleave', onLeave);
    cvs.addEventListener('wheel', onWheel, { passive: false });

    return () => {
      cvs.removeEventListener('pointerdown', onDown);
      cvs.removeEventListener('pointermove', onMove);
      cvs.removeEventListener('pointerup', onUp);
      cvs.removeEventListener('pointercancel', onUp);
      cvs.removeEventListener('pointerleave', onLeave);
      cvs.removeEventListener('wheel', onWheel);
    };
  }, [engineReady]);

  const cursorClass =
    activeTool === 'hand'     ? 'cursor-grab' :
    activeTool === 'select'   ? 'cursor-default' :
    activeTool === 'eraser'   ? 'cursor-none' :
    'cursor-crosshair';

  return (
    <div 
      ref={containerRef}
      className={`relative w-screen h-screen overflow-hidden font-brand ${darkMode ? 'dark bg-background' : 'bg-[#FAFAF8]'}`}
    >
      {/* Eraser cursor overlay */}
      {activeTool === 'eraser' && eraserPos && (
        <div
          className="pointer-events-none absolute z-20"
          style={{
            left: eraserPos.x - 12,
            top: eraserPos.y - 12,
            width: 24,
            height: 24,
          }}
        >
          <svg viewBox="0 0 40 40" width={24} height={24}>
            <circle cx="20" cy="20" r="18" fill={darkMode ? 'rgba(255,255,255,0.12)' : 'rgba(0,0,0,0.08)'} stroke={darkMode ? 'rgba(255,255,255,0.5)' : 'rgba(0,0,0,0.4)'} strokeWidth="1.5" strokeDasharray="3 2"/>
            <line x1="12" y1="20" x2="28" y2="20" stroke={darkMode ? 'rgba(255,255,255,0.5)' : 'rgba(0,0,0,0.4)'} strokeWidth="1.5" strokeLinecap="round"/>
            <line x1="20" y1="12" x2="20" y2="28" stroke={darkMode ? 'rgba(255,255,255,0.5)' : 'rgba(0,0,0,0.4)'} strokeWidth="1.5" strokeLinecap="round"/>
          </svg>
        </div>
      )}

      <canvas 
        id="aerial-canvas" 
        ref={canvasRef} 
        tabIndex={-1}
        className={`absolute inset-0 z-0 outline-none focus:outline-none touch-none ${cursorClass}`}
        onContextMenu={(e) => e.preventDefault()}
      />

      <div className="absolute top-4 left-0 right-0 z-10 flex flex-col items-center gap-2 pointer-events-none">
        <div className="flex items-center gap-2 bg-surface/80 backdrop-blur-md p-1 rounded-xl shadow-lg pointer-events-auto">
          <ToolButton icon={<MousePointer2 size={18} />} active={activeTool === 'select'} onClick={() => selectTool('select')} />
          <ToolButton icon={<Hand size={18} />} active={activeTool === 'hand'} onClick={() => selectTool('hand')} />
          <div className="w-px h-6 bg-hairline mx-1" />
          <ToolButton icon={<Pen size={18} />} active={activeTool === 'freedraw'} onClick={() => selectTool('freedraw')} />
          <ToolButton icon={<Wand2 size={18} />} active={activeTool === 'laser_pen'} onClick={() => selectTool('laser_pen')} />
          <ToolButton icon={<Highlighter size={18} />} active={activeTool === 'highlighter'} onClick={() => selectTool('highlighter')} />
          <ToolButton icon={<Eraser size={18} />} active={activeTool === 'eraser'} onClick={() => selectTool('eraser')} />
          <div className="w-px h-6 bg-hairline mx-1" />
          <ToolButton icon={<Square size={18} />} active={activeTool === 'rectangle'} onClick={() => selectTool('rectangle')} />
          <ToolButton icon={<Circle size={18} />} active={activeTool === 'ellipse'} onClick={() => selectTool('ellipse')} />
          <ToolButton icon={<Minus size={18} />} active={activeTool === 'line'} onClick={() => selectTool('line')} />
          <ToolButton icon={<ArrowUpRight size={18} />} active={activeTool === 'arrow'} onClick={() => selectTool('arrow')} />
        </div>
        
        {/* Tool Settings Palette */}
        {showColorPicker && activeTool !== 'select' && activeTool !== 'hand' && (
          <div className="flex flex-col gap-2 bg-surface/80 backdrop-blur-md p-2 rounded-xl shadow-lg pointer-events-auto animate-in fade-in slide-in-from-top-2">
            
            {/* Color Palette */}
            {activeTool !== 'eraser' && (
              <div className="flex items-center gap-1">
                {STROKE_COLORS.map((color) => {
                  const displayColor = (darkMode && color === '#1a1a2e') ? '#FAFAFA' : color;
                  return (
                    <button
                      key={color}
                      onClick={() => selectColor(color)}
                      className={`w-6 h-6 rounded-md shadow-sm transition-transform ${strokeColor === color ? 'scale-110 ring-2 ring-foreground ring-offset-1 ring-offset-background' : 'hover:scale-105'}`}
                      style={{ backgroundColor: displayColor }}
                    />
                  );
                })}
              </div>
            )}

            {/* Stroke Width Slider */}
            <div className="flex items-center gap-2 px-1 py-1">
              <span className="text-xs font-mono text-muted-foreground w-6">{strokeWidth}px</span>
              <input 
                type="range" 
                min="1" 
                max="24" 
                step="0.5" 
                value={strokeWidth}
                onChange={(e) => selectStrokeWidth(parseFloat(e.target.value))}
                className="w-24 accent-foreground"
              />
            </div>
          </div>
        )}
      </div>

      <div className="absolute bottom-4 left-4 z-10 pointer-events-auto flex items-center gap-2">
        <div className="h-10 px-3 flex items-center justify-center rounded-xl bg-surface/80 backdrop-blur-md shadow-lg pointer-events-none select-none">
          <span className="font-rephen text-lg text-foreground/80 translate-y-[2px]">aerial</span>
        </div>
        <button 
          onClick={toggleDarkMode}
          className="w-10 h-10 flex items-center justify-center rounded-xl bg-surface/80 backdrop-blur-md text-muted-foreground hover:text-foreground transition-colors cursor-pointer shadow-lg"
        >
          {darkMode ? <Sun size={18} /> : <Moon size={18} />}
        </button>
        <button 
          onClick={() => engineRef.current?.clear_board()}
          title="Clear Board"
          className="w-10 h-10 flex items-center justify-center rounded-xl bg-surface/80 backdrop-blur-md text-muted-foreground hover:text-destructive transition-colors cursor-pointer shadow-lg"
        >
          <Trash2 size={18} />
        </button>
      </div>
      
      <div className="absolute bottom-4 right-4 z-10 pointer-events-auto flex gap-1 bg-surface/80 backdrop-blur-md p-1 rounded-xl shadow-lg">
        <button onClick={() => engineRef.current?.zoom_out()} className="w-8 h-8 flex items-center justify-center text-muted-foreground hover:text-foreground rounded-lg hover:bg-white/5 cursor-pointer">
          <ZoomOut size={16} />
        </button>
        <button onClick={() => engineRef.current?.reset_view()} className="font-mono text-[11px] font-bold px-2 text-foreground flex items-center hover:bg-white/5 rounded-lg cursor-pointer">
          100%
        </button>
        <button onClick={() => engineRef.current?.zoom_in()} className="w-8 h-8 flex items-center justify-center text-muted-foreground hover:text-foreground rounded-lg hover:bg-white/5 cursor-pointer">
          <ZoomIn size={16} />
        </button>
      </div>

      {!engineReady && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-background">
          <span className="font-rephen text-6xl text-foreground animate-pulse">Aerial</span>
        </div>
      )}
    </div>
  );
}

function ToolButton({ icon, active, onClick }: { icon: React.ReactNode, active: boolean, onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className={`w-9 h-9 rounded-lg flex items-center justify-center transition-all cursor-pointer ${
        active 
          ? 'bg-foreground text-background shadow-sm' 
          : 'text-foreground/70 hover:text-foreground hover:bg-foreground/10 border border-transparent'
      }`}
    >
      {icon}
    </button>
  );
}
