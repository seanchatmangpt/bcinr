import React from 'react';

export const Legend: React.FC = () => {
  const standings = [
    { label: 'MEASURED', color: '#1f77b4' },
    { label: 'PROPOSED', color: '#9467bd' },
    { label: 'ADMITTED', color: '#17becf' },
    { label: 'CERTIFIED', color: '#2ca02c' },
    { label: 'ACTIVE', color: '#ff7f0e' },
    { label: 'FROZEN', color: '#7f7f7f' },
  ];

  return (
    <div style={{
      position: 'absolute',
      bottom: 20,
      right: 20,
      background: 'rgba(10, 10, 15, 0.9)',
      border: '1px solid rgba(0, 255, 255, 0.3)',
      padding: '15px',
      color: '#0ff',
      fontFamily: '"Courier New", Courier, monospace',
      zIndex: 100,
      boxShadow: '0 0 15px rgba(0,255,255,0.2)',
      borderRadius: '4px',
      backdropFilter: 'blur(4px)'
    }}>
      <h3 style={{ margin: '0 0 10px 0', fontSize: '14px', textTransform: 'uppercase', letterSpacing: '1px' }}>Measurement Standing</h3>
      {standings.map(s => (
        <div key={s.label} style={{ display: 'flex', alignItems: 'center', marginBottom: '8px' }}>
          <div style={{ width: '12px', height: '12px', backgroundColor: s.color, marginRight: '10px', boxShadow: `0 0 8px ${s.color}` }} />
          <span style={{ fontSize: '12px', letterSpacing: '0.5px' }}>{s.label}</span>
        </div>
      ))}
      <div style={{ marginTop: '15px', borderTop: '1px solid rgba(0,255,255,0.3)', paddingTop: '10px' }}>
        <h3 style={{ margin: '0 0 8px 0', fontSize: '12px', textTransform: 'uppercase' }}>Learner State (Borders)</h3>
        <div style={{ fontSize: '11px', marginBottom: '4px' }}>━ Solid Glow: ACTIVE</div>
        <div style={{ fontSize: '11px', marginBottom: '4px' }}>╍ Dashed Pulse: CANDIDATE</div>
        <div style={{ fontSize: '11px', opacity: 0.5 }}>× Crossed/Dim: GATED</div>
      </div>
      <div style={{ marginTop: '10px', borderTop: '1px solid rgba(0,255,255,0.3)', paddingTop: '10px' }}>
         <h3 style={{ margin: '0 0 8px 0', fontSize: '12px', textTransform: 'uppercase' }}>Other</h3>
         <div style={{ fontSize: '11px', marginBottom: '4px' }}>/// Hatch: Warning Status</div>
         <div style={{ fontSize: '11px' }}>γ: Distinguishability</div>
      </div>
    </div>
  );
};
