const canvas = document.getElementById('canvas');
if (canvas instanceof HTMLCanvasElement) {
  const width = canvas.width;
  const height = canvas.height;

  let ctx = canvas.getContext('2d');
  if (ctx) {
    ctx.beginPath();
    ctx.arc(width / 2, height / 2, 5, 0, Math.PI * 2);
    ctx.stroke();
  }
}
