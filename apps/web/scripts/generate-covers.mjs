/** Rebuild original, local vector artwork and web-ready thumbnails. No network or DB access. */
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import sharp from 'sharp';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const briefs = JSON.parse(await readFile(path.join(root, 'src/lib/article-covers.json'), 'utf8'));
const output = path.join(root, 'public/images/articles');
await mkdir(output, { recursive: true });
const escape = (s) => String(s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&apos;' }[c]));
const palettes = {
  storage: ['#dbeafe', '#2563eb', '#142d54'], pages: ['#e0e7ff', '#6366f1', '#273457'],
  network: ['#dcefe9', '#087f73', '#153f42'], blocks: ['#e6e4fa', '#6455bf', '#302b53'],
  flow: ['#dfeaff', '#3568c4', '#19315c'], cycle: ['#e0eef0', '#227b8c', '#193e4c'],
  target: ['#dfede5', '#21815b', '#214636'], split: ['#e3eafa', '#4c65a0', '#283952'],
  checklist: ['#e0f0eb', '#19826b', '#174d42'], map: ['#dfeaf7', '#356cc0', '#203c62'],
  code: ['#dfe7f5', '#47699f', '#21334d'], layers: ['#e9e3f5', '#7957ad', '#3a2e52'],
  matrix: ['#e9eaf8', '#6764b7', '#37335e'],
};
const rect = (x,y,w,h,fill,stroke='none',r=12) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" fill="${fill}" stroke="${stroke}" stroke-width="2"/>`;
const text = (x,y,s,size=20,color='#24344f',weight=500) => `<text x="${x}" y="${y}" font-family="Arial, sans-serif" font-size="${size}" font-weight="${weight}" fill="${color}">${escape(s)}</text>`;
const line = (x1,y1,x2,y2,c,dash='') => `<path d="M${x1} ${y1}L${x2} ${y2}" stroke="${c}" stroke-width="3" fill="none" ${dash ? `stroke-dasharray="${dash}"` : ''}/>`;
const circle = (x,y,r,fill,stroke='none') => `<circle cx="${x}" cy="${y}" r="${r}" fill="${fill}" stroke="${stroke}" stroke-width="3"/>`;
const check = (x,y,c) => `<path d="M${x} ${y}l9 9 19-22" fill="none" stroke="${c}" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>`;

function illustration(kind, labels, bg, accent, ink, seed) {
  const node = (x,y,label,active=false) => rect(x,y,220,76,active ? ink : '#ffffff',active ? ink : `${accent}55`) + text(x+18,y+45,label,17,active ? '#ffffff' : ink,600);
  let s = '';
  if (kind === 'storage' || kind === 'pages') {
    for(let i=2;i>=0;i--) s += rect(565+i*24,205-i*22,370,220,i===0?'#ffffff':bg,accent);
    for(let y=0;y<4;y++) for(let x=0;x<6;x++) s+=rect(590+x*50,230+y*41,40,29,(x+y+seed)%5===0?accent:bg,'none',4);
    s+=node(110,255,labels[0])+line(330,293,545,293,accent,'8 8')+text(590,463,labels[2],19,ink,600);
  } else if(kind==='matrix') {
    s+=rect(505,196,480,260,'#ffffff',`${accent}44`);
    for(let y=0;y<4;y++) for(let x=0;x<7;x++) {const on=(x*3+y+seed)%4!==0;s+=rect(530+x*62,220+y*53,48,37,on?bg:accent,'none',5);if(!on)s+=circle(554+x*62,238+y*53,5,'#ffffff');}
    s+=text(140,252,'TRUE',32,ink,700)+text(140,302,'FALSE',32,accent,700)+line(140,330,400,330,accent)+text(140,369,labels[2],19,ink);
  } else if(kind==='code') {
    s+=rect(135,180,700,300,ink)+circle(162,204,5,'#fca5a5')+circle(183,204,5,'#fde68a')+circle(204,204,5,'#86efac');
    for(let y=0;y<6;y++){s+=text(157,248+y*35,String(y+1).padStart(2,'0'),14,'#92a6c3');s+=rect(200+(y%3)*22,234+y*35,100+(seed+y*73)%350,9,y===3?'#86efac':'#a5b9da','none',4);}
    s+=rect(760,305,275,111,'#ffffff',`${accent}66`)+check(783,353,accent)+text(827,359,labels[1],18,ink,700);
  } else if(kind==='checklist') {
    s+=rect(265,190,560,280,bg,accent)+rect(245,170,560,280,'#ffffff',`${accent}55`);
    labels.forEach((l,i)=>{s+=rect(277,203+i*76,35,35,bg,'none',7)+check(281,218+i*76,accent)+text(335,228+i*76,l,21,ink,600)+line(335,246+i*76,740,246+i*76,bg);});
    s+=circle(895,328,63,accent)+check(875,330,'#ffffff');
  } else if(kind==='target') {
    for(let r=130;r>0;r-=32)s+=circle(745,315,r,r%2?bg:'#ffffff',`${accent}55`);
    s+=circle(745,315,16,accent)+line(745,315,910,191,ink)+node(140,235,labels[0])+node(140,335,labels[1],true)+line(360,372,575,315,accent,'7 7');
  } else if(kind==='layers') {
    for(let i=4;i>=0;i--){const x=300+i*42,y=180+(4-i)*43;s+=rect(x,y,490-i*44,48,i===0?ink:i===4?accent:'#ffffff',`${accent}55`,8);s+=text(x+17,y+31,['COMPONENT','INTEGRATION','SYSTEM','SYSTEM INT.','ACCEPTANCE'][i],17,i===0||i===4?'#ffffff':ink,600);}
  } else if(kind==='blocks') {
    for(let i=0;i<3;i++){const x=180+i*250,y=230+(i%2)*70;s+=rect(x+14,y+16,210,120,`${accent}33`)+rect(x,y,210,120,'#ffffff',accent)+rect(x+22,y-22,58,36,accent,'none',6)+text(x+19,y+68,labels[i],18,ink,700);if(i<2)s+=line(x+210,y+60,x+250,y+60,accent,'5 5');}
  } else if(kind==='network') {
    const points=[[260,224],[870,224],[260,410],[870,410]];
    points.forEach(([x,y])=>{s+=line(600,315,x,y,accent,'7 7')+circle(x,y,42,'#ffffff',`${accent}77`)+rect(x-13,y-13,26,26,bg,accent,6);});
    s+=rect(465,260,270,110,ink)+text(489,324,labels[1],23,'#ffffff',700)+text(162,159,labels[0],18,ink,600)+text(800,484,labels[2],18,ink,600);
  } else if(kind==='split') {
    s+=rect(135,200,380,240,'#ffffff',`${accent}44`)+rect(660,200,380,240,ink);
    s+=text(163,244,labels[0],21,ink,700)+text(690,244,labels[2],21,'#ffffff',700);
    for(let i=0;i<4;i++){s+=rect(165,277+i*32,170+(i%2)*100,11,bg,'none',4)+rect(690,277+i*32,170+(i%2)*100,11,`${accent}aa`,'none',4);}
    s+=circle(588,321,43,accent)+text(561,333,'↔',38,'#ffffff');
  } else if(kind==='cycle') {
    s+=`<path d="M270 290C270 100 940 100 940 290M940 370C940 540 270 540 270 370" stroke="${accent}" fill="none" stroke-width="3" stroke-dasharray="9 8"/>`;
    labels.forEach((l,i)=>{s+=node(150+i*300,290,l,i===1);});
    s+=circle(270,290,6,accent)+circle(940,370,6,accent);
  } else if(kind==='map') {
    for(let i=0;i<6;i++)s+=line(120+i*170,190,170+i*170,450,`${accent}22`)+line(110,210+i*44,1080,195+i*44,`${accent}22`);
    s+=`<path d="M240 350H435V250H725V365H945" stroke="${accent}" fill="none" stroke-width="7" stroke-linecap="round" stroke-linejoin="round"/>`;
    [[240,350],[590,250],[945,365]].forEach(([x,y],i)=>{s+=circle(x,y,25,i===1?ink:'#ffffff',accent)+circle(x,y,7,accent)+text(x-65,y+63,labels[i],17,ink,700);});
  } else {
    labels.forEach((l,i)=>{s+=node(150+i*300,280,l,i===1);if(i<2)s+=line(375+i*300,318,440+i*300,318,accent,'6 6');});
    s+=text(170,245,'01',17,accent)+text(470,245,'02',17,accent)+text(770,245,'03',17,accent);
  }
  return s;
}

for (const [index, [slug, brief]] of Object.entries(briefs).entries()) {
  const [bg, accent, ink] = palettes[brief.diagram];
  const titleSize = brief.title.length > 20 ? 39 : brief.title.length > 15 ? 47 : 60;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="675" viewBox="0 0 1200 675"><title>${escape(brief.title)}</title><desc>${escape(brief.subtitle)}</desc>
    <defs><pattern id="grid" width="32" height="32" patternUnits="userSpaceOnUse"><circle cx="1" cy="1" r="1" fill="${accent}" opacity=".15"/></pattern></defs>
    ${rect(0,0,1200,675,bg,'none',0)}<rect width="1200" height="675" fill="url(#grid)"/>
    ${text(54,51,'ANTBLOG / ENGINEERING NOTES',14,ink,600)}${text(1080,51,String(index+1).padStart(2,'0'),16,ink)}
    ${line(54,73,1146,73,`${accent}33`)}
    ${illustration(brief.diagram,brief.labels,bg,accent,ink,index)}
    ${rect(0,507,1200,168,ink,'none',0)}${rect(54,539,5,81,accent,'none',0)}
    ${text(78,578,brief.title,titleSize,'#ffffff',700)}${text(80,620,brief.subtitle,20,'#cbd5e1')}
  </svg>`;
  await writeFile(path.join(output, `${slug}.svg`), svg);
  await sharp(Buffer.from(svg)).webp({quality:86}).toFile(path.join(output, `${slug}.webp`));
}
console.log(`Generated ${Object.keys(briefs).length} SVG originals and WebP covers.`);
