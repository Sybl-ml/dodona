<template>
  <b-container fluid="md">
    <b-row>
      <b-col>
        <h1>Settings</h1>
        <hr>
        <h5>Change Avatar Icon:</h5>
        <avatar-upload @upload="onUpload"/>
        <b-button
          size="sm"
          variant="ready"
          type="submit"
          style="width:10rem"
          v-b-tooltip.hover
          @click="uploadAvatar"
          :disabled="!avatarSrc"
        >
          Update
        </b-button> 
      </b-col>
    </b-row>
  </b-container>
</template>

<style>
</style>
<script>
import AvatarUpload from "@/components/AvatarUpload.vue";

export default {
  name: "Settings",
  data() {
    return {
      user_data: {},
      avatarSrc: "",
    }
  },
  components: {
    AvatarUpload,
  },
  async mounted() {
    let user_id = $cookies.get("token");
    try {
      let data = await this.$http.get(
        `api/users`
      );
      this.user_data = data.data
    } catch (err) {
      console.log(err);
    }
  },
  methods: {
    onUpload(avatarSrc){
      this.avatarSrc = avatarSrc;
    },
    uploadAvatar(){
      if (this.avatarSrc) {
        this.$http.post("api/users/avatar", {
          avatar: this.avatarSrc.split(",")[1],
        });
      }
      window.location.reload()
    },
  },
};
</script>
