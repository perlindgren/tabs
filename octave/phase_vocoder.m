fs = 1000; % sample frequncy
x = 1:fs; % sample size
fft_w = 1:(fs / 2);

% Inspired by section 3.2.1 in
% Pitch-shifting algorithm design and applications in music.

f1 = 80;
f2 = f1 * 3;
f3 = f1 * 5;

data = zeros(1, fs);
t1 = (sin(2 * pi * (1:100) * f1 / fs)) / 2;
t2 = (sin(2 * pi * (1:100) * f2 / fs)) / 2;
t3 = (sin(2 * pi * (1:100) * f3 / fs)) / 2;

data(101:200) = t1;
data(301:400) = t2;
data(501:600) = t1 + t2;
data(701:800) = t2 + t3;

fft_data = fft(data);

figure(1);
clf;
hold on;
plot(x, data, '-b');
hold off;

figure(2);
clf;
hold on;
plot(fft_w, abs(fft_data(fft_w)) / (fs / 2), '-b');
hold off;
