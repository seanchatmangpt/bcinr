import React, { useMemo } from 'react';
import DeckGL from '@deck.gl/react';
import { ScatterplotLayer, TextLayer } from '@deck.gl/layers';
import type { DataPoint } from './mockData';

interface ObservatoryMapProps {
  data: DataPoint[];
  qValue: number;
  width: string;
  height: string;
}

const INITIAL_VIEW_STATE = {
  longitude: 0,
  latitude: 0,
  zoom: 5,
  pitch: 45,
  bearing: 0
};

const getColor = (standing: string): [number, number, number] => {
  switch (standing) {
    case 'MEASURED': return [31, 119, 180];
    case 'PROPOSED': return [148, 103, 189];
    case 'ADMITTED': return [23, 190, 207];
    case 'CERTIFIED': return [44, 160, 44];
    case 'ACTIVE': return [255, 127, 14];
    case 'FROZEN': return [127, 127, 127];
    default: return [255, 255, 255];
  }
};

export const ObservatoryMap: React.FC<ObservatoryMapProps> = ({ data, qValue, width, height }) => {
  const layerData = useMemo(() => data.filter(d => d.q === qValue), [data, qValue]);

  // Cyberpunk map style points
  const scatterLayer = new ScatterplotLayer<DataPoint>({
    id: `scatter-${qValue}`,
    data: layerData,
    pickable: true,
    opacity: 1, // Opacity is handled in getFillColor
    stroked: true,
    filled: true,
    radiusScale: 10000, // Scaling for map view
    radiusMinPixels: 20,
    radiusMaxPixels: 100,
    lineWidthMinPixels: 2,
    getPosition: d => [d.position[0], d.position[1]],
    getRadius: 0.5,
    getFillColor: d => {
      // Intensity: log(1+kappa)
      const intensity = Math.max(0.2, Math.min(1, Math.log1p(d.kappa) / 5)); 
      const baseCol = getColor(d.measurementStanding);
      // Opacity maps to n_eff / confidence
      const alpha = Math.min(255, Math.max(50, (d.n_eff / 100) * 255));
      return [baseCol[0] * intensity, baseCol[1] * intensity, baseCol[2] * intensity, alpha];
    },
    getLineColor: d => {
      const baseCol = getColor(d.measurementStanding);
      if (d.learnerState === 'GATED') return [80, 80, 80, 150]; // Dimmed
      if (d.learnerState === 'CANDIDATE') return [255, 255, 255, 255]; // Dashed/Pulsing representation
      return [baseCol[0], baseCol[1], baseCol[2], 255]; // ACTIVE glow
    },
    getLineWidth: d => d.learnerState === 'ACTIVE' ? 5 : (d.learnerState === 'GATED' ? 1 : 2),
  });

  const textLayer = new TextLayer<DataPoint>({
    id: `text-${qValue}`,
    data: layerData,
    getPosition: d => [d.position[0], d.position[1]],
    getText: d => `${d.status !== 'VALID' ? '/// WARN\n' : ''}γ: ${d.gamma_min_plus.toFixed(2)}`,
    getSize: 12,
    getColor: [0, 255, 255, 255],
    getAlignmentBaseline: 'center',
    fontWeight: 'bold'
  });

  return (
    <div style={{ position: 'relative', width, height, border: '1px solid rgba(0,255,255,0.2)', backgroundColor: '#050510' }}>
      <div style={{ position: 'absolute', top: 10, left: 10, zIndex: 10, color: '#0ff', fontFamily: 'monospace', fontSize: '14px', textShadow: '0 0 5px #0ff' }}>
        {qValue < 0 ? `q < 0 (Sparse)` : qValue === 0 ? `q = 0 (Coverage)` : qValue === 1 ? `q = 1 (Proportional)` : `q > 1 (Concentration)`}
      </div>
      <DeckGL
        initialViewState={INITIAL_VIEW_STATE}
        controller={true}
        layers={[scatterLayer, textLayer]}
        style={{ width: '100%', height: '100%' }}
      />
    </div>
  );
};
