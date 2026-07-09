import type {
  AiModelBillingMode,
  AiModelModality,
  AiProviderKind
} from "../api/admin";

export interface ProductProviderModelTemplate {
  provider_kind?: AiProviderKind | "all";
  provider_model: string;
  name?: string;
  modality: AiModelModality;
  billing_mode?: AiModelBillingMode;
  pricing_config?: Record<string, unknown>;
  metadata?: Record<string, unknown>;
}

interface ProductAiBillingConfig {
  walletAdjustment: {
    defaultReasons: {
      credit: string;
      debit: string;
    };
    quickAmounts: {
      credit: number[];
      debit: number[];
    };
  };
  jobActionReasons: {
    retryPoll: string;
    retryCache: string;
    failRelease: string;
    refund: string;
  };
  providerModelTemplates: ProductProviderModelTemplate[];
}

const imageMimeTypes = ["image/jpeg", "image/png", "image/webp"];
const videoMimeTypes = ["video/mp4", "video/quicktime", "video/webm"];
const audioMimeTypes = ["audio/mpeg", "audio/wav", "audio/mp4", "audio/x-m4a"];

export const productAiBillingConfig: ProductAiBillingConfig = {
  walletAdjustment: {
    defaultReasons: {
      credit: "影像生成余额充值",
      debit: "影像生成余额扣减"
    },
    quickAmounts: {
      credit: [50, 100, 300, 1000],
      debit: [20, 50, 100]
    }
  },
  jobActionReasons: {
    retryPoll: "重新查询生成任务状态",
    retryCache: "重新缓存生成素材",
    failRelease: "生成失败释放预扣",
    refund: "生成异常人工退款"
  },
  providerModelTemplates: [
    {
      provider_kind: "wuyin_keji",
      provider_model: "NanoBanana2",
      name: "旗舰图片生成",
      modality: "image",
      billing_mode: "per_image",
      pricing_config: {
        submit_path: "/api/async/image_nanoBanana2",
        capabilities: {
          ratios: ["auto", "1:1", "16:9", "9:16", "4:3", "3:4", "3:2", "2:3", "21:9"],
          resolutions: ["1K", "2K", "4K"],
          image_counts: [1],
          max_images: 1,
          inputModes: ["text", "image"],
          maxReferenceImages: 14,
          acceptedMimeTypes: imageMimeTypes,
          maxAssetSizeMb: 50
        }
      },
      metadata: {
        product_category: "image",
        recommended: true
      }
    },
    {
      provider_kind: "wuyin_keji",
      provider_model: "video_seedance",
      name: "多模态视频生成",
      modality: "video",
      billing_mode: "video_per_second",
      pricing_config: {
        submit_path: "/api/async/video_seedance",
        capabilities: {
          ratios: ["adaptive", "16:9", "9:16", "4:3", "1:1", "3:4", "21:9"],
          resolutions: ["480p", "720p"],
          durations: [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
          default_duration_seconds: 5,
          inputModes: ["text", "image", "frames", "video", "audio"],
          maxReferenceImages: 9,
          maxReferenceVideos: 3,
          maxReferenceAudios: 3,
          supportsReferenceVideo: true,
          supportsReferenceAudio: true,
          supportsFirstFrame: true,
          supportsLastFrame: true,
          acceptedMimeTypes: [...imageMimeTypes, ...videoMimeTypes, ...audioMimeTypes],
          maxAssetSizeMb: 50,
          maxImageAssetSizeMb: 30,
          maxVideoAssetSizeMb: 50,
          maxAudioAssetSizeMb: 15,
          minReferenceVideoSeconds: 2,
          maxReferenceVideoSeconds: 15,
          totalReferenceVideoSeconds: 15,
          minReferenceAudioSeconds: 2,
          maxReferenceAudioSeconds: 15,
          totalReferenceAudioSeconds: 15
        }
      },
      metadata: {
        product_category: "video",
        recommended: true
      }
    },
    {
      provider_kind: "wuyin_keji",
      provider_model: "video_digital_humans",
      name: "数字人口播生成",
      modality: "video",
      billing_mode: "video_per_second",
      pricing_config: {
        submit_path: "/api/async/video_digital_humans",
        capabilities: {
          inputModes: ["video", "audio"],
          maxReferenceVideos: 1,
          maxReferenceAudios: 1,
          supportsReferenceVideo: true,
          supportsReferenceAudio: true,
          acceptedMimeTypes: [...videoMimeTypes, ...audioMimeTypes],
          maxAssetSizeMb: 200
        }
      },
      metadata: {
        product_category: "avatar_video",
        recommended: true
      }
    },
    {
      provider_kind: "wuyin_keji",
      provider_model: "audio_tts",
      name: "文本转语音",
      modality: "audio",
      billing_mode: "audio_per_request",
      pricing_config: {
        submit_path: "/api/async/audio_tts",
        capabilities: {
          inputModes: ["text"],
          acceptedMimeTypes: audioMimeTypes,
          maxAssetSizeMb: 100
        }
      },
      metadata: {
        product_category: "audio",
        recommended: true
      }
    },
    {
      provider_kind: "wuyin_keji",
      provider_model: "voice_clone",
      name: "声音克隆",
      modality: "audio",
      billing_mode: "audio_per_request",
      pricing_config: {
        submit_path: "/api/voice/clone",
        capabilities: {
          inputModes: ["audio"],
          maxReferenceAudios: 1,
          supportsReferenceAudio: true,
          acceptedMimeTypes: audioMimeTypes,
          maxAssetSizeMb: 100,
          sync: true
        }
      },
      metadata: {
        product_category: "voice_clone",
        recommended: false
      }
    }
  ]
};
