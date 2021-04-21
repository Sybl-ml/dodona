<template>
  <b-container>
    <b-row class="justify-content-center">
      <b-form-file
        class="mb-3"
        placeholder="Choose a profile photo (Optional)"
        drop-placeholder="Drop file here..."
        accept="image/*"
        @change="handleImage"
        v-if="!hasImage"
      />

      <b-button
        class="mb-3"
        v-if="hasImage"
        variant="outline-warning"
        @click="clearImage"
        >Clear</b-button
      >
    </b-row>
    <b-row class="justify-content-center">
      <b-avatar v-if="imageSrc" :src="imageSrc" size="6rem" />
      <b-avatar
        v-else-if="original"
        :src="'data:image/png;base64,' + original"
        size="6rem"
      />
    </b-row>
  </b-container>
</template>

<script>
const base64Encode = (data) =>
  new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(data);
    reader.onload = () => resolve(reader.result);
    reader.onerror = (error) => reject(error);
  });

export default {
  data() {
    return {
      image: null,
      imageSrc: "",
    };
  },
  computed: {
    hasImage() {
      return !!this.image;
    },
    original() {
      return this.$store.state.user_data.user_data.avatar;
    },
  },
  methods: {
    handleImage(e) {
      this.image = e.target.files[0];
      base64Encode(this.image)
        .then((value) => {
          if (this.image.size > 32000) {
            alert(
              `Avatar images must be less than 32KB, image was ${this.image
                .size / 1000} KB`
            );
            this.image = null;
            return;
          }

          this.imageSrc = value;
          this.$emit("upload", this.imageSrc);
        })
        .catch(() => {
          this.imageSrc = null;
        });
    },
    clearImage() {
      this.image = null;
      this.imageSrc = "";
      this.$emit("upload", null);
    },
  },
};
</script>
