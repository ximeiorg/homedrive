import React from "react";
import { Modal, ModalContent, ModalBody, Button } from "@heroui/react";

// SVG Icons
const Icons = {
  close: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M18 6 6 18M6 6l12 12" />
    </svg>
  ),
  play: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor" stroke="none">
      <polygon points="5 3 19 12 5 21 5 3" />
    </svg>
  ),
  pause: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor" stroke="none">
      <rect x="6" y="4" width="4" height="16" />
      <rect x="14" y="4" width="4" height="16" />
    </svg>
  ),
  volumeOn: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
      <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07" />
    </svg>
  ),
  volumeOff: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
      <line x1="23" x2="1" y1="9" y2="9" />
      <path d="M22 7.5L13.5 16" stroke="currentColor" />
      <path d="M16.5 7.5L22 13" stroke="currentColor" />
    </svg>
  ),
  fullscreen: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3" />
    </svg>
  ),
};

interface VideoPlayerModalProps {
  isOpen: boolean;
  onClose: () => void;
  videoUrl: string;
  title: string;
}

export function VideoPlayerModal({ isOpen, onClose, videoUrl, title }: VideoPlayerModalProps) {
  const videoRef = React.useRef<HTMLVideoElement>(null);
  const [isPlaying, setIsPlaying] = React.useState(false);
  const [currentTime, setCurrentTime] = React.useState(0);
  const [duration, setDuration] = React.useState(0);
  const [volume, setVolume] = React.useState(1);
  const [isMuted, setIsMuted] = React.useState(false);
  const [showControls, setShowControls] = React.useState(true);
  const [isReady, setIsReady] = React.useState(false);

  // 重置状态当视频URL变化时
  React.useEffect(() => {
    if (isOpen) {
      setIsPlaying(false);
      setCurrentTime(0);
      setDuration(0);
      setVolume(1);
      setIsMuted(false);
      setIsReady(false);
    }
  }, [isOpen, videoUrl]);

  // 视频事件处理
  const handleTimeUpdate = () => {
    const video = videoRef.current;
    if (video) {
      setCurrentTime(video.currentTime);
    }
  };

  const handleLoadedMetadata = () => {
    const video = videoRef.current;
    if (video) {
      setDuration(video.duration);
      setIsReady(true);
    }
  };

  const handleEnded = () => {
    setIsPlaying(false);
    setCurrentTime(0);
  };

  // 播放控制
  React.useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    if (isPlaying && isReady) {
      video.play().catch(() => {});
    }
  }, [isPlaying, isReady]);

  const togglePlay = () => {
    if (!isReady) return;
    setIsPlaying(!isPlaying);
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const video = videoRef.current;
    if (!video) return;
    const time = parseFloat(e.target.value);
    video.currentTime = time;
    setCurrentTime(time);
  };

  const toggleMute = () => {
    const video = videoRef.current;
    if (!video) return;
    video.muted = !isMuted;
    setIsMuted(!isMuted);
  };

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const video = videoRef.current;
    if (!video) return;
    const vol = parseFloat(e.target.value);
    video.volume = vol;
    setVolume(vol);
    setIsMuted(vol === 0);
  };

  const toggleFullscreen = () => {
    const container = document.querySelector(".video-player-container");
    if (!container) return;
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      container.requestFullscreen();
    }
  };

  const formatTime = (seconds: number) => {
    if (!seconds || !Number.isFinite(seconds)) return "0:00";
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      size="full"
      classNames={{
        base: "video-player-modal",
        closeButton: "hidden",
      }}
    >
      <ModalContent>
        <ModalBody 
          className="p-0 bg-black"
          onMouseEnter={() => setShowControls(true)}
          onMouseLeave={() => setShowControls(false)}
        >
          <div className="video-player-container relative w-full h-screen flex items-center justify-center bg-black">
            {/* Close button */}
            <Button
              isIconOnly
              variant="light"
              className="absolute top-4 right-4 z-50 bg-black/50 hover:bg-black/70 text-white"
              onPress={onClose}
            >
              <span className="w-6 h-6"><Icons.close /></span>
            </Button>

            {/* Video */}
            <video
              ref={videoRef}
              src={videoUrl}
              className="max-w-full max-h-full w-full h-full object-contain"
              onClick={togglePlay}
              onTimeUpdate={handleTimeUpdate}
              onLoadedMetadata={handleLoadedMetadata}
              onEnded={handleEnded}
              onPlay={() => setIsPlaying(true)}
              onPause={() => setIsPlaying(false)}
            />

            {/* Loading indicator */}
            {!isReady && (
              <div className="absolute inset-0 flex items-center justify-center">
                <div className="w-12 h-12 border-4 border-white/30 border-t-white rounded-full animate-spin" />
              </div>
            )}

            {/* Play/Pause overlay */}
            {isReady && !isPlaying && (
              <div 
                className="absolute inset-0 flex items-center justify-center cursor-pointer"
                onClick={togglePlay}
              >
                <div className="w-24 h-24 rounded-full bg-white/20 backdrop-blur-sm flex items-center justify-center">
                  <span className="w-14 h-14 text-white"><Icons.play /></span>
                </div>
              </div>
            )}

            {/* Controls */}
            <div 
              className={`absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-4 transition-opacity duration-300 ${
                showControls ? "opacity-100" : "opacity-0"
              }`}
            >
              {/* Progress bar */}
              <div className="mb-4">
                <input
                  type="range"
                  min={0}
                  max={duration > 0 ? duration : 100}
                  step={0.1}
                  value={currentTime}
                  onChange={handleSeek}
                  disabled={!isReady}
                  className="w-full h-1 bg-white/30 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-primary disabled:[&::-webkit-slider-thumb]:bg-white/50"
                />
              </div>

              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  {/* Play/Pause */}
                  <Button
                    isIconOnly
                    variant="light"
                    className="text-white disabled:opacity-50"
                    onPress={togglePlay}
                    isDisabled={!isReady}
                  >
                    {isPlaying ? (
                      <span className="w-6 h-6"><Icons.pause /></span>
                    ) : (
                      <span className="w-6 h-6"><Icons.play /></span>
                    )}
                  </Button>

                  {/* Volume */}
                  <div className="flex items-center gap-2">
                    <Button
                      isIconOnly
                      variant="light"
                      className="text-white"
                      onPress={toggleMute}
                      isDisabled={!isReady}
                    >
                      {isMuted || volume === 0 ? (
                        <span className="w-5 h-5"><Icons.volumeOff /></span>
                      ) : (
                        <span className="w-5 h-5"><Icons.volumeOn /></span>
                      )}
                    </Button>
                    <input
                      type="range"
                      min={0}
                      max={1}
                      step={0.1}
                      value={isMuted ? 0 : volume}
                      onChange={handleVolumeChange}
                      disabled={!isReady}
                      className="w-20 h-1 bg-white/30 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-primary disabled:[&::-webkit-slider-thumb]:bg-white/50"
                    />
                  </div>

                  {/* Time */}
                  <span className="text-white text-sm">
                    {formatTime(currentTime)} / {formatTime(duration)}
                  </span>
                </div>

                <div className="flex items-center gap-2">
                  {/* Title */}
                  <span className="text-white text-sm truncate max-w-xs">{title}</span>
                  
                  {/* Fullscreen */}
                  <Button
                    isIconOnly
                    variant="light"
                    className="text-white"
                    onPress={toggleFullscreen}
                  >
                    <span className="w-5 h-5"><Icons.fullscreen /></span>
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </ModalBody>
      </ModalContent>
    </Modal>
  );
}
