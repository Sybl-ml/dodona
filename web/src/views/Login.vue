<template>
  <b-container fluid>
    <b-row class="justify-content-center text-center">
      <b-col lg="4" md="6" sm="8" xs="12">
        <icon-logo
          class="mt-5 mb-3"
          height="10em"
          width="10em"
          :show_text="false"
        />
        <h1 class="mb-3"><strong>Please Sign in</strong></h1>
        <b-card bordered-variant="primary" class="text-center mt-3 mb-5">
          <b-form class="mt-3 mb-3" @submit.prevent="onSubmit">
            <b-input-group class="mb-3" prepend="@">
              <b-form-input
                type="email"
                required
                placeholder="Enter Email"
                v-model="email"
              />
            </b-input-group>

            <b-input-group class="mb-3" prepend="#">
              <b-form-input
                :type="passwordType"
                required
                placeholder="Password"
                v-model="password"
                pattern="^.{1,32}$"
                title="Password must contain at least eight characters"
              />
              <template #append>
                <b-input-group-text>
                  <b-icon
                    style="cursor: pointer;"
                    :icon="passwordIcon"
                    @click="hidePassword = !hidePassword"
                  />
                </b-input-group-text>
              </template>
            </b-input-group>

            <b-form-checkbox
              style="display:none;"
              value="me"
              class="mb-3"
              v-model="remember_password"
              >Remember Me</b-form-checkbox
            >
            <b-button class="mb-3" variant="primary" type="submit" block>
              SIGN IN
              <b-spinner v-show="submitted" small></b-spinner>
            </b-button>
            <a href="/forgot">Forgotten Password?</a>
          </b-form>
        </b-card>
      </b-col>
    </b-row>
    <b-row class="justify-content-center text-center">
      <b-alert v-model="failed" variant="danger" dismissible>
        Incorrect Username or Password
      </b-alert>
    </b-row>
  </b-container>
</template>

<script>
import IconLogo from "@/components/icons/IconLogo";

export default {
  data() {
    return {
      email: "",
      password: "",
      remember_password: false,
      hidePassword: true,
      submitted: false,
      failed: false,
    };
  },
  components: {
    IconLogo,
  },
  computed: {
    passwordType() {
      return this.hidePassword ? "password" : "text";
    },
    passwordIcon() {
      return this.hidePassword ? "eye-fill" : "eye-slash-fill";
    },
  },
  methods: {
    sleep(ms) {
      return new Promise((resolve) => {
        setTimeout(resolve, ms);
      });
    },
    async onSubmit() {
      this.submitted = true;

      let response = await this.$store.dispatch("login", {
        email: this.email,
        password: this.password,
      });

      $cookies.set("token", response.data.token, {
        path: "/",
        sameSite: true,
      });

      this.$router.push("dashboard");
      this.failed = true;
      this.submitted = false;
    },
  },
};
</script>
