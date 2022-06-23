from django.db import models
from datetime import datetime


class Email(models.Model):
	author = models.CharField('From:', max_length=50)
	subject = models.CharField('Subject', max_length=50)
	message = models.TextField('Message')
	date = models.DateTimeField(default=datetime.now, blank=True)

	def __str__(self):
		return self.subject
